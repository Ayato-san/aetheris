//! Native Winsock/IP Helper adapter. windows-sys only supplies Windows bindings.
//! SIO_RCVALL: https://learn.microsoft.com/windows/win32/winsock/sio-rcvall
use super::Interface;
use crate::{
    capture::{
        CaptureBatch, CaptureSource, CapturedPacket, LinkType, PacketTimestamp, Result, network,
    },
    game_process::is_game_executable,
};
use std::{
    collections::{BTreeMap, VecDeque},
    ffi::c_void,
    mem::size_of,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    ptr,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, ERROR_NO_MORE_FILES, GetLastError, INVALID_HANDLE_VALUE},
    NetworkManagement::IpHelper::*,
    Networking::WinSock::*,
    System::Diagnostics::ToolHelp::*,
};

pub(super) mod firewall;
pub(super) mod identity;

struct Winsock;
impl Winsock {
    fn new() -> Result<Self> {
        let mut data = WSADATA::default();
        // A valid output buffer is supplied and this guard balances WSAStartup.
        let code = unsafe { WSAStartup(0x0202, &mut data) };
        if code != 0 {
            return Err(format!("Initialisation Winsock : {code}").into());
        }
        Ok(Self)
    }
}
impl Drop for Winsock {
    fn drop(&mut self) {
        unsafe {
            WSACleanup();
        }
    }
}

struct RawSocket {
    socket: SOCKET,
    interface: Interface,
}
impl RawSocket {
    fn open(interface: Interface) -> Result<Self> {
        let handle = unsafe { socket(AF_INET as i32, SOCK_RAW, IPPROTO_IP) };
        if handle == INVALID_SOCKET {
            return Err(winsock_error("Ouverture de la capture"));
        }
        let raw = Self {
            socket: handle,
            interface,
        };
        let mut address = SOCKADDR_IN {
            sin_family: AF_INET,
            ..Default::default()
        };
        address.sin_addr.S_un.S_addr = u32::from_ne_bytes(raw.interface.address.octets());
        // The address and option buffers remain valid for the synchronous calls.
        let result = unsafe {
            bind(
                handle,
                (&address as *const SOCKADDR_IN).cast(),
                size_of::<SOCKADDR_IN>() as i32,
            )
        };
        if result == SOCKET_ERROR {
            return Err(winsock_error("Liaison à l’interface"));
        }
        let mode = RCVALL_IPLEVEL;
        let mut returned = 0;
        let result = unsafe {
            WSAIoctl(
                handle,
                SIO_RCVALL,
                (&mode as *const RCVALL_VALUE).cast(),
                size_of_val(&mode) as u32,
                ptr::null_mut(),
                0,
                &mut returned,
                ptr::null_mut(),
                None,
            )
        };
        if result == SOCKET_ERROR {
            return Err(winsock_error("Activation SIO_RCVALL"));
        }
        let mut nonblocking = 1;
        if unsafe { ioctlsocket(handle, FIONBIO, &mut nonblocking) } == SOCKET_ERROR {
            return Err(winsock_error("Mode non bloquant"));
        }
        Ok(raw)
    }
}
impl Drop for RawSocket {
    fn drop(&mut self) {
        let mode = RCVALL_OFF;
        let mut returned = 0;
        unsafe {
            WSAIoctl(
                self.socket,
                SIO_RCVALL,
                (&mode as *const RCVALL_VALUE).cast(),
                size_of_val(&mode) as u32,
                ptr::null_mut(),
                0,
                &mut returned,
                ptr::null_mut(),
                None,
            );
            closesocket(self.socket);
        }
    }
}

fn winsock_error(context: &str) -> Box<dyn std::error::Error + Send + Sync> {
    let code = unsafe { WSAGetLastError() };
    if code == WSAEACCES {
        format!("{context} : droits administrateur requis. Fermez puis relancez Aetheris en tant qu’administrateur.").into()
    } else {
        format!("{context} : erreur Winsock {code}.").into()
    }
}

pub(super) fn check_access() -> Result<()> {
    let _winsock = Winsock::new()?;
    let handle = unsafe { socket(AF_INET as i32, SOCK_RAW, IPPROTO_IP) };
    if handle == INVALID_SOCKET {
        return Err(winsock_error("Capture réseau"));
    }
    unsafe {
        closesocket(handle);
    }
    Ok(())
}

pub(super) fn interfaces() -> Result<Vec<Interface>> {
    let mut size = 16_384u32;
    for _ in 0..4 {
        // u64 storage provides sufficient alignment for the linked native structures.
        let mut buffer = vec![0u64; (size as usize).div_ceil(8)];
        let result = unsafe {
            GetAdaptersAddresses(
                AF_INET as u32,
                GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER,
                ptr::null(),
                buffer.as_mut_ptr().cast(),
                &mut size,
            )
        };
        if result == 111 {
            continue;
        } // ERROR_BUFFER_OVERFLOW; retry resized buffer.
        if result != 0 {
            return Err(format!("Lecture des interfaces : erreur Windows {result}").into());
        }
        let mut interfaces = Vec::new();
        let mut adapter = buffer.as_mut_ptr().cast::<IP_ADAPTER_ADDRESSES_LH>();
        // Windows links all nodes and strings inside the owned buffer above.
        unsafe {
            while !adapter.is_null() {
                let entry = &*adapter;
                if entry.OperStatus == 1 {
                    let name = wide_string(entry.FriendlyName);
                    let mut unicast = entry.FirstUnicastAddress;
                    while !unicast.is_null() {
                        let address = (*unicast).Address;
                        if !address.lpSockaddr.is_null()
                            && (*address.lpSockaddr).sa_family == AF_INET
                            && address.iSockaddrLength as usize >= size_of::<SOCKADDR_IN>()
                        {
                            let address = &*address.lpSockaddr.cast::<SOCKADDR_IN>();
                            let ip = Ipv4Addr::from(address.sin_addr.S_un.S_addr.to_ne_bytes());
                            if !ip.is_unspecified() {
                                interfaces.push(Interface {
                                    name: name.clone(),
                                    address: ip,
                                    index: entry.Anonymous1.Anonymous.IfIndex,
                                });
                            }
                        }
                        unicast = (*unicast).Next;
                    }
                }
                adapter = entry.Next;
            }
        }
        return Ok(interfaces);
    }
    Err("Les interfaces réseau changent trop rapidement ; réessayez.".into())
}

unsafe fn wide_string(pointer: *const u16) -> String {
    if pointer.is_null() {
        return String::new();
    }
    let mut length = 0;
    // Native FriendlyName is a null-terminated string owned by the adapter buffer.
    unsafe {
        while *pointer.add(length) != 0 {
            length += 1;
        }
        String::from_utf16_lossy(std::slice::from_raw_parts(pointer, length))
    }
}

fn game_pid(owner: &identity::Identity) -> Result<Option<identity::Process>> {
    let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if handle == INVALID_HANDLE_VALUE {
        return Err("Impossible de lire la liste des processus Windows.".into());
    }
    struct Snapshot(*mut c_void);
    impl Drop for Snapshot {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
    let snapshot = Snapshot(handle);
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut available = unsafe { Process32FirstW(snapshot.0, &mut entry) };
    let mut found = None;
    loop {
        if available == 0 {
            if unsafe { GetLastError() } != ERROR_NO_MORE_FILES {
                return Err("Liste des processus Windows incomplète.".into());
            }
            break;
        }
        let length = entry
            .szExeFile
            .iter()
            .position(|value| *value == 0)
            .unwrap_or(entry.szExeFile.len());
        if is_game_executable(&String::from_utf16_lossy(&entry.szExeFile[..length]))
            && identity::session(entry.th32ProcessID).is_ok_and(|session| session == owner.session)
        {
            let process = identity::Process::open(entry.th32ProcessID)?;
            // UAC may launch AION under a separate administrator account in
            // the client's interactive session. Its SID need not match the
            // authenticated capture client; other sessions remain excluded.
            if process.identity.session == owner.session {
                if found.is_some() {
                    return Err(
                        "Plusieurs processus AION 2 détectés : attachement suspendu.".into(),
                    );
                }
                found = Some(process);
            }
        }
        available = unsafe { Process32NextW(snapshot.0, &mut entry) };
    }
    Ok(found)
}

fn native_table<T: Copy>(mut query: impl FnMut(*mut c_void, &mut u32) -> u32) -> Result<Vec<T>> {
    let mut size = 0;
    let result = query(ptr::null_mut(), &mut size);
    if result != 0 && result != 122 {
        return Err(format!("Lecture des sockets : erreur Windows {result}").into());
    }
    for _ in 0..4 {
        if size < 4 {
            return Ok(Vec::new());
        }
        if size > 64 * 1024 * 1024 {
            return Err("Table des sockets anormalement grande.".into());
        }
        let mut buffer = vec![0u64; (size as usize).div_ceil(8)];
        let result = query(buffer.as_mut_ptr().cast(), &mut size);
        if result == 122 {
            continue;
        }
        if result != 0 {
            return Err(format!("Lecture des sockets : erreur Windows {result}").into());
        }
        let count = unsafe { ptr::read_unaligned(buffer.as_ptr().cast::<u32>()) } as usize;
        if count.saturating_mul(size_of::<T>()).saturating_add(4) > size as usize {
            return Err("Table des sockets tronquée.".into());
        }
        return Ok((0..count)
            .map(|index| unsafe {
                ptr::read_unaligned(
                    buffer
                        .as_ptr()
                        .cast::<u8>()
                        .add(4 + index * size_of::<T>())
                        .cast::<T>(),
                )
            })
            .collect());
    }
    Err("Les sockets changent trop rapidement ; réessayez.".into())
}

#[derive(Default)]
struct Owners {
    tcp: BTreeMap<(SocketAddr, SocketAddr), Instant>,
    udp: BTreeMap<SocketAddr, Instant>,
    ipv6_present: bool,
    shared_udp: bool,
}

impl Owners {
    fn refresh(&mut self, pid: u32) -> Result<()> {
        let now = Instant::now();
        // A tuple no longer reported for the PID must not be attributed to it
        // merely because another process reused that tuple shortly afterwards.
        self.tcp.clear();
        self.udp.clear();
        self.shared_udp = false;
        let tcp = native_table::<MIB_TCPROW_OWNER_PID>(|buffer, length| unsafe {
            GetExtendedTcpTable(
                buffer,
                length,
                0,
                AF_INET as u32,
                TCP_TABLE_OWNER_PID_ALL,
                0,
            )
        })?;
        for row in tcp
            .into_iter()
            .filter(|row| row.dwOwningPid == pid && row.dwRemotePort != 0)
        {
            let local = endpoint(row.dwLocalAddr, row.dwLocalPort);
            let remote = endpoint(row.dwRemoteAddr, row.dwRemotePort);
            self.tcp.insert((local, remote), now);
            self.tcp.insert((remote, local), now);
        }
        let udp = native_table::<MIB_UDPROW_OWNER_PID>(|buffer, length| unsafe {
            GetExtendedUdpTable(buffer, length, 0, AF_INET as u32, UDP_TABLE_OWNER_PID, 0)
        })?;
        for row in udp.iter().filter(|row| row.dwOwningPid == pid) {
            if !exclusive_udp(row, &udp) {
                self.shared_udp = true;
                continue;
            }
            self.udp
                .insert(endpoint(row.dwLocalAddr, row.dwLocalPort), now);
        }
        let tcp6 = native_table::<MIB_TCP6ROW_OWNER_PID>(|buffer, length| unsafe {
            GetExtendedTcpTable(
                buffer,
                length,
                0,
                AF_INET6 as u32,
                TCP_TABLE_OWNER_PID_ALL,
                0,
            )
        })?;
        let udp6 = native_table::<MIB_UDP6ROW_OWNER_PID>(|buffer, length| unsafe {
            GetExtendedUdpTable(buffer, length, 0, AF_INET6 as u32, UDP_TABLE_OWNER_PID, 0)
        })?;
        self.ipv6_present = tcp6.iter().any(|row| row.dwOwningPid == pid)
            || udp6.iter().any(|row| row.dwOwningPid == pid);
        Ok(())
    }

    fn matches(&self, info: &network::PacketInfo) -> bool {
        let (Some(source), Some(destination)) = (info.source, info.destination) else {
            return false;
        };
        if info.protocol == "TCP" {
            return self.tcp.contains_key(&(source, destination));
        }
        if info.protocol == "UDP" {
            return self.udp.keys().any(|local| {
                [source, destination].iter().any(|endpoint| {
                    local.port() == endpoint.port()
                        && (local.ip().is_unspecified() || local.ip() == endpoint.ip())
                })
            });
        }
        false
    }
}

fn exclusive_udp(row: &MIB_UDPROW_OWNER_PID, all: &[MIB_UDPROW_OWNER_PID]) -> bool {
    !all.iter().any(|other| {
        other.dwOwningPid != row.dwOwningPid
            && other.dwLocalPort == row.dwLocalPort
            && (row.dwLocalAddr == 0
                || other.dwLocalAddr == 0
                || row.dwLocalAddr == other.dwLocalAddr)
    })
}

fn endpoint(address: u32, port: u32) -> SocketAddr {
    SocketAddr::new(
        IpAddr::V4(Ipv4Addr::from(address.to_ne_bytes())),
        u16::from_be(port as u16),
    )
}

/// Resolve the owner of an established local IPC endpoint from Windows, rather
/// than trusting identity fields supplied over that connection.
pub(super) fn tcp_owner(local: SocketAddr, remote: SocketAddr) -> Result<u32> {
    let rows = native_table::<MIB_TCPROW_OWNER_PID>(|buffer, length| unsafe {
        GetExtendedTcpTable(
            buffer,
            length,
            0,
            AF_INET as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        )
    })?;
    rows.into_iter()
        .find(|row| {
            row.dwState == MIB_TCP_STATE_ESTAB as u32
                && endpoint(row.dwLocalAddr, row.dwLocalPort) == local
                && endpoint(row.dwRemoteAddr, row.dwRemotePort) == remote
        })
        .map(|row| row.dwOwningPid)
        .ok_or_else(|| "Propriétaire de la connexion locale introuvable.".into())
}

pub(super) struct NetworkSource {
    sockets: Vec<RawSocket>, // Closed before the Winsock guard.
    _winsock: Winsock,
    manual: Option<Interface>,
    pid: Option<u32>,
    owner: identity::Identity,
    // Hold the authenticated game process object across socket-table refreshes
    // so its PID cannot be recycled into an unrelated process's attribution.
    game: Option<identity::Process>,
    section: u32,
    owners: Owners,
    last_refresh: Instant,
    pending: VecDeque<(Instant, CapturedPacket)>,
    next_number: u64,
    status: String,
    fragments: BTreeMap<(Ipv4Addr, Ipv4Addr, u16, u8), Instant>,
}

impl NetworkSource {
    pub(super) fn new(manual: Option<Interface>, owner: identity::Identity) -> Result<Self> {
        Ok(Self {
            sockets: Vec::new(),
            _winsock: Winsock::new()?,
            manual,
            pid: None,
            owner,
            game: None,
            section: 0,
            owners: Owners::default(),
            last_refresh: Instant::now() - Duration::from_secs(1),
            pending: VecDeque::new(),
            next_number: 1,
            status: "En attente d’AION 2".into(),
            fragments: BTreeMap::new(),
        })
    }

    fn refresh(&mut self) -> Result<()> {
        let game = match game_pid(&self.owner) {
            Ok(game) => game,
            Err(error) => {
                self.status = error.to_string();
                self.sockets.clear();
                self.pending.clear();
                self.owners = Owners::default();
                self.fragments.clear();
                self.pid = None;
                self.game = None;
                return Ok(());
            }
        };
        let pid = game.as_ref().map(|game| game.pid);
        self.game = game;
        if pid != self.pid {
            self.section += 1;
            self.owners = Owners::default();
            self.fragments.clear();
            // A pending SYN can precede process discovery. Retain it only when
            // moving from no client to a single client, never across replacements.
            if self.pid.is_some() {
                self.pending.clear();
            }
            self.pid = pid;
        }
        if let Some(pid) = pid {
            self.owners.refresh(pid)?;
        }
        let available = interfaces()?;
        let desired = if let Some(interface) = &self.manual {
            vec![interface.clone()]
        } else {
            available
                .into_iter()
                .filter(|interface| {
                    self.owners
                        .tcp
                        .keys()
                        .any(|(local, _)| local.ip() == IpAddr::V4(interface.address))
                        || self
                            .owners
                            .udp
                            .keys()
                            .any(|local| local.ip() == IpAddr::V4(interface.address))
                })
                .collect()
        };
        self.sockets.retain(|socket| {
            desired
                .iter()
                .any(|interface| interface.address == socket.interface.address)
        });
        for interface in desired {
            if !self
                .sockets
                .iter()
                .any(|socket| socket.interface.address == interface.address)
            {
                self.sockets.push(RawSocket::open(interface)?);
            }
        }
        self.status = match pid {
            None => "En attente d’AION 2".into(),
            Some(_) if self.sockets.is_empty() =>
                "En attente de sockets IPv4 ; choisissez une interface pour capturer avant la connexion".into(),
            Some(pid) => format!("Capture IPv4 — PID {pid} · {} interface(s)",self.sockets.len()),
        };
        if self.owners.ipv6_present {
            self.status
                .push_str(" · IPv6 live indisponible : en-tête original non restitué par Winsock");
        }
        if self.owners.shared_udp {
            self.status
                .push_str(" · UDP partagé avec un autre processus : attribution suspendue");
        }
        Ok(())
    }
}

impl CaptureSource for NetworkSource {
    fn poll(&mut self) -> Result<CaptureBatch> {
        if self.last_refresh.elapsed() >= Duration::from_millis(100) {
            self.refresh()?;
            self.last_refresh = Instant::now();
        }
        let mut capture_error = None;
        'interfaces: for socket in &self.sockets {
            let mut buffer = [0u8; 65535];
            for _ in 0..512 {
                let length =
                    unsafe { recv(socket.socket, buffer.as_mut_ptr(), buffer.len() as i32, 0) };
                if length == SOCKET_ERROR {
                    if unsafe { WSAGetLastError() } == WSAEWOULDBLOCK {
                        break;
                    }
                    capture_error = Some(winsock_error("Lecture de la capture").to_string());
                    break 'interfaces;
                }
                let bytes = buffer[..length as usize].to_vec();
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .ok()
                    .and_then(|duration| u64::try_from(duration.as_nanos()).ok());
                self.pending.push_back((
                    Instant::now(),
                    CapturedPacket {
                        number: 0,
                        section_id: self.section,
                        interface_id: socket.interface.index,
                        local_address: Some(IpAddr::V4(socket.interface.address)),
                        link_type: LinkType::RawIpv4,
                        timestamp_ns: timestamp.map(i128::from),
                        timestamp: timestamp.map(|ticks| PacketTimestamp {
                            ticks,
                            resolution: 9,
                            offset_seconds: 0,
                        }),
                        captured_len: length as u32,
                        original_len: length as u32,
                        bytes,
                    },
                ));
            }
        }
        let mut packets = Vec::new();
        let mut waiting = VecDeque::new();
        // Associate fragments through an observed first-fragment tuple owned
        // by the game. They remain raw; this is not IP reassembly.
        self.fragments
            .retain(|_, seen| seen.elapsed() < Duration::from_secs(5));
        if self.pid.is_some() {
            for (_, packet) in &self.pending {
                if let Some((key, Some(first))) = fragment(packet)
                    && self.owners.matches(&first)
                {
                    self.fragments.insert(key, Instant::now());
                }
            }
        }
        while let Some((received, mut packet)) = self.pending.pop_front() {
            let owned_fragment =
                fragment(&packet).is_some_and(|(key, _)| self.fragments.contains_key(&key));
            if self.pid.is_some()
                && (self.owners.matches(&network::parse(&packet)) || owned_fragment)
            {
                packet.number = self.next_number;
                packet.section_id = self.section;
                self.next_number += 1;
                packets.push(packet);
            } else if received.elapsed() < Duration::from_millis(750) {
                waiting.push_back((received, packet));
            }
        }
        self.pending = waiting;
        if self
            .pending
            .iter()
            .map(|(_, packet)| packet.bytes.len())
            .sum::<usize>()
            > 16 * 1024 * 1024
        {
            return Ok(CaptureBatch {
                packets, status: self.status.clone(), finished: true,
                error: Some("Tampon d’attribution des sockets saturé (16 Mio) : capture arrêtée, données déjà reçues conservées.".into()),
            });
        }
        Ok(CaptureBatch {
            packets,
            status: self.status.clone(),
            finished: capture_error.is_some(),
            error: capture_error,
        })
    }
}

type FragmentKey = (Ipv4Addr, Ipv4Addr, u16, u8);

fn fragment(packet: &CapturedPacket) -> Option<(FragmentKey, Option<network::PacketInfo>)> {
    let bytes = &packet.bytes;
    if bytes.len() < 20 || bytes[0] >> 4 != 4 {
        return None;
    }
    let flags = u16::from_be_bytes([bytes[6], bytes[7]]);
    if flags & 0x3fff == 0 {
        return None;
    }
    let source = Ipv4Addr::new(bytes[12], bytes[13], bytes[14], bytes[15]);
    let destination = Ipv4Addr::new(bytes[16], bytes[17], bytes[18], bytes[19]);
    let key = (
        source,
        destination,
        u16::from_be_bytes([bytes[4], bytes[5]]),
        bytes[9],
    );
    let header = (bytes[0] as usize & 15) * 4;
    if flags & 0x1fff != 0 || header < 20 || bytes.len() < header + 4 {
        return Some((key, None));
    }
    Some((
        key,
        Some(network::PacketInfo {
            source: Some(SocketAddr::new(
                source.into(),
                u16::from_be_bytes([bytes[header], bytes[header + 1]]),
            )),
            destination: Some(SocketAddr::new(
                destination.into(),
                u16::from_be_bytes([bytes[header + 2], bytes[header + 3]]),
            )),
            protocol: match bytes[9] {
                6 => "TCP",
                17 => "UDP",
                _ => "IP",
            }
            .into(),
            ..Default::default()
        }),
    ))
}

#[cfg(test)]
mod ownership_tests {
    use super::*;
    #[test]
    fn udp_shared_with_another_process_is_never_disclosed_as_game_traffic() {
        let game = MIB_UDPROW_OWNER_PID {
            dwLocalAddr: 10,
            dwLocalPort: 99,
            dwOwningPid: 1,
        };
        let foreign = MIB_UDPROW_OWNER_PID {
            dwOwningPid: 2,
            ..game
        };
        assert!(exclusive_udp(&game, &[game]));
        assert!(!exclusive_udp(&game, &[game, foreign]));
        assert!(!exclusive_udp(
            &game,
            &[
                game,
                MIB_UDPROW_OWNER_PID {
                    dwLocalAddr: 0,
                    ..foreign
                }
            ]
        ));
        assert!(exclusive_udp(
            &game,
            &[
                game,
                MIB_UDPROW_OWNER_PID {
                    dwLocalAddr: 11,
                    ..foreign
                }
            ]
        ));
    }
}
