using System.Security.Cryptography;
using System.Text.Json;
using CUE4Parse.FileProvider;
using CUE4Parse.UE4.Assets.Exports.Engine;
using CUE4Parse.UE4.Assets.Exports.Paper2D;
using CUE4Parse.UE4.Assets.Exports.Texture;
using CUE4Parse.UE4.Objects.UObject;

internal static class UiSprites {
    public static void Export(DefaultFileProvider provider, string mapping, string output) {
        var sprites = new List<object>();
        var images = new List<object>();
        var errors = new List<object>();
        var paths = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);
        var files = provider.Files.Values.Select(f => f.Path)
            .Where(p => p.StartsWith("AION2/Content/UI/Resource/Sprite/", StringComparison.OrdinalIgnoreCase)
                && p.EndsWith(".uasset", StringComparison.OrdinalIgnoreCase))
            .Distinct(StringComparer.OrdinalIgnoreCase).Order().ToArray();
        var packages = 0;
        foreach (var path in files) {
            try {
                var exports = provider.LoadPackage(path).GetExports().OfType<UPaperSprite>().ToArray();
                foreach (var sprite in exports) {
                    var objectPath = sprite.GetPathName();
                    var reference = "UI/Sprite/" + path.Split("/Sprite/", 2)[1][..^7];
                    if (exports.Length > 1) reference += "/" + sprite.Name;
                    if (reference.Contains(':') || reference.Contains('\\')
                        || reference.Split('/').Any(p => p is "" or "." or ".."))
                        throw new InvalidDataException("Invalid UI sprite reference path.");
                    var decoded = IconTexture.Decode(sprite);
                    var destination = Path.Combine(output, reference + ".png");
                    Directory.CreateDirectory(Path.GetDirectoryName(destination)!);
                    File.WriteAllBytes(destination, decoded.Png);
                    sprites.Add(new { reference, asset_path = objectPath, width = decoded.Width, height = decoded.Height });
                    images.Add(new { reference, state = "available", sources = new[] { path },
                        texture_source = decoded.TextureSource, width = decoded.Width, height = decoded.Height,
                        pixel_format = decoded.PixelFormat, sha256 = Convert.ToHexStringLower(SHA256.HashData(decoded.Png)) });
                    paths.Add(objectPath, reference);
                }
            } catch (Exception error) {
                errors.Add(new { path, error = error.Message });
            }
            if (++packages % 100 == 0) Console.WriteLine($"Read {packages}/{files.Length} packages; extracted {sprites.Count} sprites.");
        }
        var aliases = new List<object>();
        foreach (var table in new[] { "UI_ImageTable_Composite", "UI_ItemIconTable_Composite", "UI_SkillIconTable" }) {
            var data = provider.LoadPackage($"AION2/Content/UnrealData/{table}.uasset")
                .GetExports().OfType<UDataTable>().Single();
            foreach (var (key, row) in data.RowMap) {
                var resource = row.GetOrDefault<FSoftObjectPath>("ResourceObject");
                if (resource.AssetPathName.Text is { } asset && paths.TryGetValue(asset, out var reference))
                    aliases.Add(new { table, key = key.Text, reference });
            }
        }
        // Popup_Item uses distinct wide textures; inventory squares come from
        // UI_ImageTable_Composite ItemGrade_* (source codes, not localized names).
        foreach (var grade in new[] { "Common", "Rare", "Legend", "Unique", "Epic", "Mythic", "Special" }) {
            foreach (var prefix in new[] { "UT_ItemTooltipGrade_", "UT_SlotGrade_" }) {
                var reference = $"UI/Resource/Texture/ETC/{prefix}{grade}";
                var path = $"AION2/Content/{reference}.uasset";
                try {
                    var texture = provider.LoadPackage(path).GetExports().OfType<UTexture2D>().Single();
                    var decoded = IconTexture.Decode(texture);
                    var destination = Path.Combine(output, reference + ".png");
                    Directory.CreateDirectory(Path.GetDirectoryName(destination)!);
                    File.WriteAllBytes(destination, decoded.Png);
                    images.Add(new { reference, state = "available", sources = new[] { path },
                        width = decoded.Width, height = decoded.Height, pixel_format = decoded.PixelFormat,
                        sha256 = Convert.ToHexStringLower(SHA256.HashData(decoded.Png)) });
                } catch (Exception error) {
                    images.Add(new { reference, state = "error", sources = new[] { path }, error = error.Message });
                }
            }
        }
        var sha = Convert.ToHexStringLower(SHA256.HashData(File.ReadAllBytes(mapping)));
        var options = new JsonSerializerOptions { WriteIndented = true };
        File.WriteAllText(Path.Combine(output, "ui-sprites.json"), JsonSerializer.Serialize(new {
            format = 1, mapping_sha256 = sha, packages = files.Length, sprites, aliases, errors,
        }, options));
        var reportPath = Path.Combine(output, "extraction.json");
        var combined = new List<JsonElement>();
        if (File.Exists(reportPath)) {
            using var previous = JsonDocument.Parse(File.ReadAllBytes(reportPath));
            if (previous.RootElement.GetProperty("mapping_sha256").GetString() != sha)
                throw new InvalidDataException("Cannot combine incompatible sprite and icon mappings.");
            combined.AddRange(previous.RootElement.GetProperty("results").EnumerateArray().Select(e => e.Clone()));
        }
        combined.AddRange(images.Select(i => JsonSerializer.SerializeToElement(i)));
        var results = combined.GroupBy(r => r.GetProperty("reference").GetString()).Select(g => g.Last()).ToArray();
        File.WriteAllText(reportPath, JsonSerializer.Serialize(new {
            mapping_sha256 = sha, requested = results.Length,
            exported = results.Count(r => r.GetProperty("state").GetString() == "available"), results,
        }, options));
        Console.WriteLine($"Extracted {sprites.Count} sprites, {aliases.Count} UI keys; {errors.Count} errors.");
    }
}
