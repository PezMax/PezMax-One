using System;
using System.Text;
using System.Threading.Tasks;
using PezMax.Login;
using PezMax.Shell;
using Windows.ApplicationModel;
using Windows.ApplicationModel.Activation;
using Windows.Storage;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Navigation;

namespace PezMax
{
    public sealed partial class App : Application
    {
        public App()
        {
            UnhandledException += (_, e) => WriteCrashLog(e.Exception);
            AppDomain.CurrentDomain.UnhandledException += (_, e) => WriteCrashLog(e.ExceptionObject as Exception);
            TaskScheduler.UnobservedTaskException += (_, e) => WriteCrashLog(e.Exception);
            try
            {
                InitializeComponent();
            }
            catch (Exception ex)
            {
                WriteCrashLog(ex);
                throw;
            }

            Suspending += OnSuspending;
        }

        public static void WriteLog(string fileName, string text)
        {
            try
            {
                var path = System.IO.Path.Combine(ApplicationData.Current.LocalFolder.Path, fileName);
                System.IO.File.AppendAllText(path, text);
            }
            catch
            {
            }

            try
            {
                var path = System.IO.Path.Combine(System.IO.Path.GetTempPath(), "pezmax-" + fileName);
                System.IO.File.AppendAllText(path, text);
            }
            catch
            {
            }

            try
            {
                var path = System.IO.Path.Combine(
                    Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "pezmax-" + fileName);
                System.IO.File.AppendAllText(path, text);
            }
            catch
            {
            }
        }

        public static void WriteCrashLog(Exception? ex)
        {
            try
            {
                var sb = new StringBuilder();
                sb.AppendLine($"--- {DateTimeOffset.Now:O} ---");
                sb.AppendLine($"{ex?.GetType().FullName}: {ex?.Message}");
                sb.AppendLine(ex?.StackTrace);
                WriteLog("crash.log", sb.ToString());
            }
            catch
            {
            }
        }

        protected override async void OnLaunched(LaunchActivatedEventArgs e)
        {
            var rootFrame = Window.Current.Content as Frame;
            if (rootFrame == null)
            {
                rootFrame = new Frame();
                rootFrame.NavigationFailed += OnNavigationFailed;
                Window.Current.Content = rootFrame;
            }

            try
            {
                await AppServices.InitializeAsync();
                AppServices.ApplyTheme(rootFrame);
            }
            catch (Exception ex)
            {
                WriteCrashLog(ex);
            }

            var restored = false;
            try
            {
                restored = await AppServices.TryRestoreSessionAsync();
            }
            catch (Exception ex)
            {
                WriteCrashLog(ex);
            }

            if (!e.PrelaunchActivated && rootFrame.Content == null)
            {
                rootFrame.Navigate(restored ? typeof(ShellPage) : typeof(LoginPage), e.Arguments);
            }

            Window.Current.Activate();
        }

        private static void OnNavigationFailed(object sender, NavigationFailedEventArgs e)
        {
            var ex = new Exception($"无法导航到页面 {e.SourcePageType?.FullName}", e.Exception);
            WriteCrashLog(ex);
            throw ex;
        }

        private static void OnSuspending(object sender, SuspendingEventArgs e)
        {
            var deferral = e.SuspendingOperation.GetDeferral();
            deferral.Complete();
        }
    }
}
