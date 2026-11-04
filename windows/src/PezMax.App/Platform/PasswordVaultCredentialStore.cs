using PezMax.Core.Platform;
using Windows.Security.Credentials;

namespace PezMax.Platform;

public sealed class PasswordVaultCredentialStore : ICredentialStore
{
    private const string Resource = "PezMax.One";
    private const string UserName = "token";

    public string? GetToken()
    {
        try
        {
            var credential = new PasswordVault().Retrieve(Resource, UserName);
            credential.RetrievePassword();
            return credential.Password;
        }
        catch
        {
            return null;
        }
    }

    public void SetToken(string token)
    {
        var vault = new PasswordVault();
        Clear(vault);
        vault.Add(new PasswordCredential(Resource, UserName, token));
    }

    public void ClearToken() => Clear(new PasswordVault());

    private static void Clear(PasswordVault vault)
    {
        try
        {
            foreach (var credential in vault.FindAllByResource(Resource))
            {
                vault.Remove(credential);
            }
        }
        catch
        {
        }
    }
}
