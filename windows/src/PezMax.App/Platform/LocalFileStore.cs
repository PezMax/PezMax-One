using System;
using System.IO;
using System.Threading.Tasks;
using PezMax.Core.Platform;
using Windows.Storage;

namespace PezMax.Platform;

public sealed class LocalFileStore : IFileStore
{
    private static string Root => ApplicationData.Current.LocalFolder.Path;

    public async Task<string?> ReadTextAsync(string relativePath)
    {
        try
        {
            var file = await ApplicationData.Current.LocalFolder.GetFileAsync(relativePath.Replace('/', '\\'));
            return await FileIO.ReadTextAsync(file);
        }
        catch (FileNotFoundException)
        {
            return null;
        }
        catch (Exception)
        {
            return null;
        }
    }

    public async Task WriteTextAsync(string relativePath, string content)
    {
        var normalized = relativePath.Replace('/', '\\');
        var folder = Path.GetDirectoryName(normalized);
        var target = ApplicationData.Current.LocalFolder;
        if (!string.IsNullOrEmpty(folder))
        {
            target = await target.CreateFolderAsync(folder, CreationCollisionOption.OpenIfExists);
        }

        var file = await target.CreateFileAsync(Path.GetFileName(normalized), CreationCollisionOption.ReplaceExisting);
        await FileIO.WriteTextAsync(file, content);
    }

    public async Task DeleteAsync(string relativePath)
    {
        try
        {
            var file = await ApplicationData.Current.LocalFolder.GetFileAsync(relativePath.Replace('/', '\\'));
            await file.DeleteAsync();
        }
        catch (Exception)
        {
        }
    }

    public async Task<bool> ExistsAsync(string relativePath)
    {
        try
        {
            await ApplicationData.Current.LocalFolder.GetFileAsync(relativePath.Replace('/', '\\'));
            return true;
        }
        catch (Exception)
        {
            return false;
        }
    }
}
