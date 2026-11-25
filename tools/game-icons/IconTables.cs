using CUE4Parse.FileProvider;
using CUE4Parse.UE4.Assets.Exports.Engine;
using CUE4Parse.UE4.Objects.UObject;

// IconRes is a data-table row key, not necessarily a texture's filename.
internal sealed record IconBinding(string Table, string ObjectPath, string PackagePath);

internal static class IconTables {
    public static Dictionary<string, IconBinding> Read(DefaultFileProvider provider) {
        var bindings = new Dictionary<string, IconBinding>(StringComparer.OrdinalIgnoreCase);
        foreach (var tablePath in new[] {
            "AION2/Content/UnrealData/UI_ItemIconTable_Composite.uasset",
            "AION2/Content/UnrealData/UI_SkillIconTable.uasset",
            "AION2/Content/UnrealData/UI_ImageTable_Composite.uasset",
        }) {
            var tables = provider.LoadPackage(tablePath).GetExports().OfType<UDataTable>().ToArray();
            if (tables.Length != 1 || tables[0].RowMap is null)
                throw new InvalidDataException($"Expected one readable icon data table: {tablePath}");
            foreach (var (key, row) in tables[0].RowMap) {
                // General UI row keys can collide with item keys. Only currency
                // references belong to this export's use of the general table.
                if (tablePath.EndsWith("/UI_ImageTable_Composite.uasset", StringComparison.Ordinal)
                    && !key.Text.StartsWith("Currency_", StringComparison.OrdinalIgnoreCase)) continue;
                var resource = row.GetOrDefault<FSoftObjectPath>("ResourceObject");
                var objectPath = resource.AssetPathName.Text;
                if (string.IsNullOrEmpty(objectPath) || objectPath == "None") continue;
                if (!string.IsNullOrEmpty(resource.SubPathString))
                    throw new InvalidDataException($"Unsupported icon subobject: {key.Text}");
                var dot = objectPath.LastIndexOf('.');
                if (!objectPath.StartsWith("/Game/", StringComparison.Ordinal) || dot < 6)
                    throw new InvalidDataException($"Unsupported icon object path: {objectPath}");
                var packagePath = "AION2/Content/" + objectPath[6..dot] + ".uasset";
                var binding = new IconBinding(tablePath, objectPath, packagePath);
                if (bindings.TryGetValue(key.Text, out var previous)
                    && !previous.ObjectPath.Equals(objectPath, StringComparison.OrdinalIgnoreCase))
                    throw new InvalidDataException($"Conflicting icon tables for {key.Text}");
                bindings[key.Text] = binding;
            }
        }
        return bindings;
    }
}
