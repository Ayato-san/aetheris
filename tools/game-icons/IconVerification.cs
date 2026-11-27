using System.Security.Cryptography;
using System.Text.Json;
using SkiaSharp;

internal static class IconVerification {
    public static void Verify(string directory) {
        var maps = !File.Exists(Path.Combine(directory, "extraction.json"));
        using var report = JsonDocument.Parse(File.ReadAllText(Path.Combine(directory, maps ? "world-maps.json" : "extraction.json")));
        var count = 0;
        foreach (var row in report.RootElement.GetProperty(maps ? "images" : "results").EnumerateArray()) {
            if (!maps && row.GetProperty("state").GetString() != "available") continue;
            var reference = row.GetProperty("reference").GetString()!;
            var bytes = File.ReadAllBytes(Path.Combine(directory, reference + ".png"));
            var sha = Convert.ToHexStringLower(SHA256.HashData(bytes));
            if (sha != row.GetProperty("sha256").GetString()) throw new InvalidDataException($"Checksum mismatch: {reference}");
            using var bitmap = SKBitmap.Decode(bytes) ?? throw new InvalidDataException($"Invalid PNG: {reference}");
            if (bitmap.Width != row.GetProperty("width").GetInt32() || bitmap.Height != row.GetProperty("height").GetInt32())
                throw new InvalidDataException($"Unexpected dimensions: {reference}");
            count++;
        }
        var expected = maps ? report.RootElement.GetProperty("images").GetArrayLength() : report.RootElement.GetProperty("exported").GetInt32();
        if (count != expected) throw new InvalidDataException("Export count mismatch.");
        Console.WriteLine($"Verified {count} PNG images, dimensions and SHA-256 checksums.");
    }

    public static void Compare(string expected, string actual) {
        var count = 0;
        foreach (var file in Directory.EnumerateFiles(expected, "*.png", SearchOption.AllDirectories)) {
            var relative = Path.GetRelativePath(expected, file);
            using var a = SKBitmap.Decode(file) ?? throw new InvalidDataException($"Invalid expected PNG: {relative}");
            using var b = SKBitmap.Decode(Path.Combine(actual, relative)) ?? throw new InvalidDataException($"Invalid actual PNG: {relative}");
            if (a.Info != b.Info || !a.GetPixelSpan().SequenceEqual(b.GetPixelSpan()))
                throw new InvalidDataException($"Pixel mismatch: {relative}");
            count++;
        }
        if (count == 0) throw new InvalidDataException("No PNG images to compare.");
        Console.WriteLine($"Verified identical decoded pixels for {count} images.");
    }
}
