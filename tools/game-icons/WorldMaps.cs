using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using CUE4Parse.FileProvider;
using CUE4Parse.UE4.Assets.Exports.Texture;
using CUE4Parse.UE4.Objects.Core.Math;
using CUE4Parse.UE4.Objects.UObject;

// Offline conversion, like icons. Runtime consumers read the PNGs and explicit
// actor/texture bindings; filenames never establish a verified map transform.
internal static class WorldMaps {
    public static void Export(DefaultFileProvider provider, string mapping, string output) {
        var levels = new List<object>();
        var images = new Dictionary<string, object>(StringComparer.Ordinal);
        var errors = new List<object>();
        var textureErrors = new List<object>();
        string ExportTexture(UTexture2D texture) {
            var objectPath = texture.GetPathName();
            var reference = "WorldMaps/Textures/" + Convert.ToHexStringLower(SHA256.HashData(Encoding.UTF8.GetBytes(objectPath)));
            if (!images.ContainsKey(reference)) {
                var decoded = IconTexture.Decode(texture);
                var relative = reference + ".png";
                var destination = Path.Combine(output, relative);
                Directory.CreateDirectory(Path.GetDirectoryName(destination)!);
                File.WriteAllBytes(destination, decoded.Png);
                images.Add(reference, new { reference, object_path = objectPath, file = relative,
                    width = decoded.Width, height = decoded.Height, pixel_format = decoded.PixelFormat,
                    sha256 = Convert.ToHexStringLower(SHA256.HashData(decoded.Png)) });
            }
            return reference;
        }
        var files = provider.Files.Values.Where(f => f.Path.StartsWith("AION2/Content/UI/Map/", StringComparison.OrdinalIgnoreCase)
            && f.Path.EndsWith(".umap", StringComparison.OrdinalIgnoreCase))
            .GroupBy(f => f.Path, StringComparer.OrdinalIgnoreCase).Select(g => {
                var first = g.First();
                if (g.Skip(1).Any(f => !SHA256.HashData(f.Read()).AsSpan().SequenceEqual(SHA256.HashData(first.Read()))))
                    throw new InvalidDataException($"Conflicting map aliases: {g.Key}");
                return first;
            }).OrderBy(f => f.Path).ToArray();
        if (files.Length > 4096) throw new InvalidDataException("Too many map levels.");
        foreach (var file in files) {
            try {
                var actors = new List<object>();
                foreach (var actor in provider.LoadPackage(file.Path).GetExports()) {
                    if (!actor.TryGetValue(out FPackageIndex textureIndex, "SectorTexture")) continue;
                    var index = actor.GetOrDefault<FIntPoint>("SectorIndex");
                    var hasIndex = actor.TryGetValue(out FIntPoint _, "SectorIndex");
                    // Missing serialized indices inherit the Blueprint default.
                    // Keep this absence explicit instead of fabricating (0,0).
                    var texture = textureIndex.Load<UTexture2D>()
                        ?? throw new InvalidDataException("Missing sector texture.");
                    var reference = ExportTexture(texture);
                    actors.Add(new { name = actor.Name, actor_class = actor.ExportType,
                        sector_x = hasIndex ? (int?)index.X : null, sector_y = hasIndex ? (int?)index.Y : null,
                        index_state = hasIndex ? "explicit" : "inherited_unresolved", texture_reference = reference });
                }
                levels.Add(new { source_path = file.Path, payload_sha256 = Convert.ToHexStringLower(SHA256.HashData(file.Read())), actors });
            } catch (Exception error) {
                errors.Add(new { source_path = file.Path, error = error.Message });
            }
            if ((levels.Count + errors.Count) % 20 == 0)
                Console.WriteLine($"Read {levels.Count + errors.Count}/{files.Length} map levels; {images.Count} textures, {errors.Count} errors.");
        }
        // Also recover assets with no actor reference in the mounted levels,
        // including shared-map floors, masks and path layers. Loading the export
        // establishes its Texture2D class; the directory does not establish a map.
        var textureFiles = provider.Files.Values.Where(f => f.Path.StartsWith("AION2/Content/UI/Map/WorldMap/", StringComparison.OrdinalIgnoreCase)
            && f.Path.EndsWith(".uasset", StringComparison.OrdinalIgnoreCase))
            .GroupBy(f => f.Path, StringComparer.OrdinalIgnoreCase).Select(g => {
                var first = g.First();
                if (g.Skip(1).Any(f => !SHA256.HashData(f.Read()).AsSpan().SequenceEqual(SHA256.HashData(first.Read()))))
                    throw new InvalidDataException($"Conflicting map texture aliases: {g.Key}");
                return first;
            }).OrderBy(f => f.Path).ToArray();
        if (textureFiles.Length > 8192) throw new InvalidDataException("Too many map assets.");
        var inspected = 0;
        foreach (var file in textureFiles) {
            try {
                foreach (var texture in provider.LoadPackage(file.Path).GetExports().OfType<UTexture2D>()) ExportTexture(texture);
            } catch (Exception error) {
                textureErrors.Add(new { source_path = file.Path, error = error.Message });
            }
            if (++inspected % 100 == 0) Console.WriteLine($"Inspected {inspected}/{textureFiles.Length} map assets; {images.Count} textures, {textureErrors.Count} asset errors.");
        }
        File.WriteAllText(Path.Combine(output, "world-maps.json"), JsonSerializer.Serialize(new {
            format = 1, mapping_sha256 = Convert.ToHexStringLower(SHA256.HashData(File.ReadAllBytes(mapping))),
            packages = files.Length, texture_packages = textureFiles.Length, levels, images = images.Values, errors, texture_errors = textureErrors,
        }, new JsonSerializerOptions { WriteIndented = true }));
        Console.WriteLine($"Extracted {images.Count} map textures from {levels.Count} levels; {errors.Count} level errors, {textureErrors.Count} asset errors.");
    }
}
