// Offline texture conversion; this executable is not part of the Aetheris client.
using System.Security.Cryptography;
using System.Text.Json;
using CUE4Parse.Compression;
using CUE4Parse.Encryption.Aes;
using CUE4Parse.FileProvider;
using CUE4Parse.MappingsProvider.Usmap;
using CUE4Parse.UE4.Assets.Exports.Texture;
using CUE4Parse.UE4.Assets.Exports.Paper2D;
using CUE4Parse.UE4.AssetRegistry;
using CUE4Parse.UE4.Objects.Core.Misc;
using CUE4Parse.UE4.Versions;

if (args.Length == 2 && args[0] == "--verify") {
    IconVerification.Verify(args[1]);
    return;
}
if (args.Length == 3 && args[0] == "--compare") {
    IconVerification.Compare(args[1], args[2]);
    return;
}
var includeSprites = args.Length == 6 && args[0] == "--include-ui-sprites";
if (includeSprites) args = args[1..];
var inspectAsset = args.Length == 6 && args[0] == "--inspect-asset" ? args[4] : null;
if (inspectAsset is not null) args = [args[1], args[2], args[3], "", args[5]];
var inventoryOnly = args.Length == 5 && args[0] == "--inventory";
var spritesOnly = args.Length == 5 && args[0] == "--ui-sprites";
var mapsOnly = args.Length == 5 && args[0] == "--world-maps";
if (inventoryOnly || spritesOnly || mapsOnly) args = [args[1], args[2], args[3], "", args[4]];
if (args.Length != 5)
    throw new ArgumentException("Usage: GameIcons <paks> <mapping.usmap> <key-file> <references.json> <output>");
var paks = Path.GetFullPath(args[0]);
var output = Path.GetFullPath(args[4]);
if (output.StartsWith(paks + Path.DirectorySeparatorChar, StringComparison.OrdinalIgnoreCase) || output == paks)
    throw new ArgumentException("Output must be outside the game archives.");
Directory.CreateDirectory(output);
if (Directory.EnumerateFileSystemEntries(output).Any())
    throw new ArgumentException("Use an empty output directory to avoid reusing stale icons.");
var references = inventoryOnly || spritesOnly || mapsOnly || inspectAsset is not null ? [] : JsonSerializer.Deserialize<string[]>(File.ReadAllText(args[3]))
    ?? throw new ArgumentException("Missing icon references.");
if (references.Length > 100000) throw new ArgumentException("Too many references.");
var keyLines = File.ReadAllLines(args[2]).Select(line => line.Trim())
    .Where(line => line.Length > 0 && !line.StartsWith('#') && !line.StartsWith("//")).ToArray();
if (keyLines.Length != 1) throw new ArgumentException("Expected one hexadecimal archive key.");
var key = keyLines[0];
if (key.StartsWith("0x", StringComparison.OrdinalIgnoreCase)) key = key[2..];
if (key.Length != 64 || !key.All(Uri.IsHexDigit)) throw new ArgumentException("Invalid archive key.");

OodleHelper.Initialize(Path.Combine(AppContext.BaseDirectory, OodleHelper.OodleFileName));
using var provider = new DefaultFileProvider(paks, SearchOption.AllDirectories,
    new VersionContainer(EGame.GAME_Aion2), StringComparer.OrdinalIgnoreCase) {
    MappingsContainer = new FileUsmapTypeMappingsProvider(args[1]),
};
provider.Initialize();
provider.SubmitKey(new FGuid(), new FAesKey("0x" + key));
provider.PostMount();
Console.WriteLine($"Mounted {provider.Files.Count} virtual files.");
if (inspectAsset is not null) {
    File.WriteAllText(Path.Combine(output, "asset.json"), Newtonsoft.Json.JsonConvert.SerializeObject(
        provider.LoadPackage(inspectAsset).GetExports(), Newtonsoft.Json.Formatting.Indented));
    return;
}
if (spritesOnly) {
    UiSprites.Export(provider, args[1], output);
    return;
}
if (mapsOnly) {
    WorldMaps.Export(provider, args[1], output);
    return;
}
if (inventoryOnly) {
    File.WriteAllText(Path.Combine(output, "inventory.json"), JsonSerializer.Serialize(new {
        mounted = provider.MountedVfs.Select(v => v.Name).Order().ToArray(),
        unloaded = provider.UnloadedVfs.Select(v => v.Name).Order().ToArray(),
        files = provider.Files.Values.Select(f => f.Path).Order().ToArray(),
    }));
    File.WriteAllText(Path.Combine(output, "configuration.json"), JsonSerializer.Serialize(
        provider.Files.Values.Where(f => f.Path.StartsWith("Aion2/Config/", StringComparison.OrdinalIgnoreCase)
            && f.Path.EndsWith(".ini", StringComparison.OrdinalIgnoreCase) && f.Size < 1024 * 1024)
            .GroupBy(f => f.Path, StringComparer.OrdinalIgnoreCase).ToDictionary(g => g.Key,
                g => System.Text.Encoding.UTF8.GetString(g.First().Read()))));
    foreach (var file in provider.Files.Values.Where(f => f.Path.EndsWith(".uasset", StringComparison.OrdinalIgnoreCase)
        && (f.Path.Contains("IconTable", StringComparison.OrdinalIgnoreCase)
            || f.Path.EndsWith("/UI_ImageTable_Composite.uasset", StringComparison.OrdinalIgnoreCase)))) {
        var package = provider.LoadPackage(file.Path);
        File.WriteAllText(Path.Combine(output, Path.GetFileNameWithoutExtension(file.Path) + ".json"),
            Newtonsoft.Json.JsonConvert.SerializeObject(package.GetExports(), Newtonsoft.Json.Formatting.Indented));
    }
    var registryIndex = 0;
    foreach (var file in provider.Files.Values.Where(f => f.Path.EndsWith("/AssetRegistry.bin", StringComparison.OrdinalIgnoreCase))) {
        try {
            using var reader = file.CreateReader();
            var registry = new FAssetRegistryState(reader);
            var assets = registry.PreallocatedAssetDataBuffers;
            File.WriteAllText(Path.Combine(output, $"registry-{registryIndex++}.json"), JsonSerializer.Serialize(new {
                source = file.Path, total = assets.Length,
                icons = assets.Where(a => a.ObjectPath.Contains("icon", StringComparison.OrdinalIgnoreCase)
                    || a.AssetClass.Text == "ObjectRedirector").Select(a => new {
                    path = a.ObjectPath, kind = a.AssetClass.Text,
                    tags = a.TagsAndValues.ToDictionary(t => t.Key.Text, t => t.Value),
                }).ToArray(),
            }));
            Console.WriteLine($"Read registry: {assets.Length} assets.");
        } catch (Exception error) {
            File.WriteAllText(Path.Combine(output, $"registry-{registryIndex++}-error.txt"), error.ToString());
            Console.WriteLine($"Registry error: {error.Message}");
        }
    }
    return;
}
var textures = provider.Files.Values.Where(f => f.Path.EndsWith(".uasset", StringComparison.OrdinalIgnoreCase))
    .Select(f => f.Path).Distinct(StringComparer.OrdinalIgnoreCase).ToArray();
var byName = textures.GroupBy(Path.GetFileNameWithoutExtension, StringComparer.OrdinalIgnoreCase)
    .ToDictionary(g => g.Key!, g => g.ToArray(), StringComparer.OrdinalIgnoreCase);
var bindings = IconTables.Read(provider);
var texturePaths = textures.ToHashSet(StringComparer.OrdinalIgnoreCase);
Console.WriteLine($"Read {bindings.Count} icon table bindings.");
var results = new List<object>();
var succeeded = 0;
IconImage Decode(string path) {
    var package = provider.LoadPackage(path);
    var images = package.GetExports().OfType<UTexture2D>().ToArray();
    var sprites = package.GetExports().OfType<UPaperSprite>().ToArray();
    if (images.Length == 0 && sprites.Length == 1) return IconTexture.Decode(sprites[0]);
    if (images.Length != 1) throw new InvalidDataException($"Expected one Texture2D, got {images.Length}.");
    return IconTexture.Decode(images[0]);
}
foreach (var reference in references.Distinct(StringComparer.Ordinal)
    .Where(r => !includeSprites || !r.StartsWith("UI/Sprite/", StringComparison.Ordinal))) {
    try {
        var relative = reference.Replace('\\', '/');
        if (relative.Split('/').Any(s => s is "" or "." or "..") || relative.Contains(':'))
            throw new ArgumentException("Invalid icon reference path.");
        var named = byName.GetValueOrDefault(Path.GetFileName(relative), []);
        var candidates = named.Where(p => p.EndsWith("/" + relative + ".uasset", StringComparison.OrdinalIgnoreCase)).ToArray();
        if (candidates.Length == 0) candidates = named;
        bindings.TryGetValue(relative, out var binding);
        // The declared object wins over basename matches, including duplicate
        // shop/ETC textures. A missing declared asset must not use a namesake.
        if (binding is not null)
            candidates = texturePaths.Contains(binding.PackagePath) ? [binding.PackagePath] : [];
        else if (relative.StartsWith("UI/Sprite/", StringComparison.Ordinal)) {
            var path = "AION2/Content/UI/Resource/Sprite/" + relative[10..] + ".uasset";
            candidates = texturePaths.Contains(path) ? [path] : [];
        }
        if (candidates.Length == 0 || candidates.Length > 16) {
            results.Add(new { reference, state = candidates.Length == 0 ? "not_found" : "ambiguous", candidates, binding });
            continue;
        }
        var decoded = Decode(candidates[0]);
        var identical = candidates.Skip(1).All(path => decoded.Png.AsSpan().SequenceEqual(Decode(path).Png));
        if (!identical) {
            results.Add(new { reference, state = "ambiguous", candidates });
            continue;
        }
        var bytes = decoded.Png;
        var destination = Path.Combine(output, relative + ".png");
        Directory.CreateDirectory(Path.GetDirectoryName(destination)!);
        var temporary = destination + ".tmp";
        File.WriteAllBytes(temporary, bytes);
        File.Move(temporary, destination, true);
        results.Add(new { reference, state = "available", sources = candidates, binding, width = decoded.Width,
            height = decoded.Height, pixel_format = decoded.PixelFormat, texture_source = decoded.TextureSource,
            sha256 = Convert.ToHexStringLower(SHA256.HashData(bytes)) });
        succeeded++;
    } catch (Exception error) {
        results.Add(new { reference, state = "error", error = error.Message });
    }
    if (results.Count % 100 == 0) Console.WriteLine($"Processed {results.Count}/{references.Length}; exported {succeeded}.");
}
var report = new { mapping_sha256 = Convert.ToHexStringLower(SHA256.HashData(File.ReadAllBytes(args[1]))),
    requested = references.Length, exported = succeeded, results };
File.WriteAllText(Path.Combine(output, "extraction.json"), JsonSerializer.Serialize(report, new JsonSerializerOptions { WriteIndented = true }));
Console.WriteLine($"Exported {succeeded}/{references.Length}; report: {Path.Combine(output, "extraction.json")}");
if (includeSprites) UiSprites.Export(provider, args[1], output);
