using System.Runtime.InteropServices;
using CUE4Parse.UE4.Assets.Exports.Texture;
using CUE4Parse.UE4.Assets.Exports.Paper2D;
using CUE4Parse_Conversion.Textures.BC;
using SkiaSharp;

internal record IconImage(int Width, int Height, string PixelFormat, byte[] Png, string? TextureSource = null);

internal static class IconTexture {
    private static readonly Dictionary<string, IconImage> SpriteSheets = new();
    private static long sheetBytes;
    public static IconImage Decode(UPaperSprite sprite) {
        var texture = sprite.BakedSourceTexture?.Load<UTexture2D>()
            ?? throw new InvalidDataException("Missing baked sprite texture.");
        var texturePath = texture.GetPathName();
        if (!SpriteSheets.TryGetValue(texturePath, out var decoded)) {
            decoded = Decode(texture);
            if (sheetBytes + decoded.Png.Length > 64 * 1024 * 1024 || SpriteSheets.Count >= 16) {
                SpriteSheets.Clear(); sheetBytes = 0;
            }
            SpriteSheets[texturePath] = decoded;
            sheetBytes += decoded.Png.Length;
        }
        using var sheet = SKBitmap.Decode(decoded.Png);
        var importedWidth = texture.ImportedSize.X > 0 ? texture.ImportedSize.X : decoded.Width;
        var importedHeight = texture.ImportedSize.Y > 0 ? texture.ImportedSize.Y : decoded.Height;
        var scaleX = (float)decoded.Width / importedWidth;
        var scaleY = (float)decoded.Height / importedHeight;
        var x = (int)MathF.Round(sprite.BakedSourceUV.X * scaleX);
        var y = (int)MathF.Round(sprite.BakedSourceUV.Y * scaleY);
        var width = (int)MathF.Round(sprite.BakedSourceDimension.X * scaleX);
        var height = (int)MathF.Round(sprite.BakedSourceDimension.Y * scaleY);
        if (width <= 0 || height <= 0 || x < 0 || y < 0 || x > sheet.Width - width || y > sheet.Height - height)
            throw new InvalidDataException("Invalid baked sprite rectangle.");
        using var crop = new SKBitmap();
        if (!sheet.ExtractSubset(crop, SKRectI.Create(x, y, width, height)))
            throw new InvalidDataException("Cannot extract sprite rectangle.");
        // Rotation is editor-only metadata; recover it from baked positions/UVs.
        var vertices = sprite.BakedRenderData;
        float Spread(Func<CUE4Parse.UE4.Objects.Core.Math.FVector4, float> offset) =>
            vertices.Max(offset) - vertices.Min(offset);
        var ppu = sprite.PixelsPerUnrealUnit;
        var rotated = vertices.Length > 0 &&
            Spread(v => v.X * ppu - v.W * importedHeight) + Spread(v => v.Y * ppu - v.Z * importedWidth)
            < Spread(v => v.X * ppu - v.Z * importedWidth) + Spread(v => v.Y * ppu + v.W * importedHeight);
        using var upright = new SKBitmap(rotated ? height : width, rotated ? width : height);
        using (var canvas = new SKCanvas(upright)) {
            canvas.Clear(SKColors.Transparent);
            if (rotated) { canvas.Translate(0, width); canvas.RotateDegrees(-90); }
            canvas.DrawBitmap(crop, 0, 0);
        }
        using var image = SKImage.FromBitmap(upright);
        using var png = image.Encode(SKEncodedImageFormat.Png, 100);
        return new IconImage(upright.Width, upright.Height, decoded.PixelFormat, png.ToArray(), texture.GetPathName());
    }
    public static IconImage Decode(UTexture2D texture) {
        if (texture.PlatformData.VTData is { } vt && vt.IsInitialized())
            throw new NotSupportedException("Virtual textures are not supported by this icon exporter.");
        var mip = texture.GetFirstMip() ?? throw new InvalidDataException("No readable texture mip.");
        var width = mip.SizeX;
        var height = mip.SizeY;
        if (width <= 0 || height <= 0 || width > 4096 || height > 4096 || mip.SizeZ != 1)
            throw new InvalidDataException("Icon dimensions exceed bounds.");
        var raw = mip.BulkData?.Data ?? throw new InvalidDataException("Missing mip bulk data.");
        var pixels = texture.Format switch {
            EPixelFormat.PF_DXT1 => BCDecoder.BC1(raw, width, height, 1),
            EPixelFormat.PF_DXT3 => BCDecoder.BC2(raw, width, height, 1),
            EPixelFormat.PF_DXT5 => BCDecoder.BC3(raw, width, height, 1),
            EPixelFormat.PF_BC4 => BCDecoder.BC4(raw, width, height, 1),
            EPixelFormat.PF_BC5 => BCDecoder.BC5(raw, width, height, 1),
            EPixelFormat.PF_BC7 => BCDecoder.BC7(raw, width, height, 1),
            EPixelFormat.PF_B8G8R8A8 or EPixelFormat.PF_R8G8B8A8 or EPixelFormat.PF_G8 => raw,
            _ => throw new NotSupportedException($"Unsupported icon pixel format: {texture.Format}"),
        };
        var color = texture.Format switch {
            EPixelFormat.PF_B8G8R8A8 or EPixelFormat.PF_BC4 or EPixelFormat.PF_BC5 => SKColorType.Bgra8888,
            EPixelFormat.PF_G8 => SKColorType.Gray8,
            _ => SKColorType.Rgba8888,
        };
        using var bitmap = new SKBitmap(new SKImageInfo(width, height, color, SKAlphaType.Unpremul));
        if (pixels.Length != bitmap.ByteCount) throw new InvalidDataException("Unexpected texture byte count.");
        Marshal.Copy(pixels, 0, bitmap.GetPixels(), pixels.Length);
        using var image = SKImage.FromBitmap(bitmap);
        using var png = image.Encode(SKEncodedImageFormat.Png, 100);
        return new IconImage(width, height, texture.Format.ToString(), png.ToArray());
    }
}
