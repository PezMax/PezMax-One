using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Navigation;

namespace PezMax.Pages
{
    public sealed partial class BrowsePage : Page
    {
        public BrowsePage()
        {
            InitializeComponent();
        }

        protected override void OnNavigatedTo(NavigationEventArgs e)
        {
            base.OnNavigatedTo(e);
            if (e.Parameter is string parameter)
            {
                if (parameter == "favorites") Tabs.SelectedIndex = 3;
                else if (parameter == "ebooks") Tabs.SelectedIndex = 1;
                else if (parameter == "bookmarks") Tabs.SelectedIndex = 2;
                else if (parameter.Length > 0) ApplySearch(parameter);
            }
        }

        public void ApplySearch(string keyword)
        {
            switch (Tabs.SelectedIndex)
            {
                case 1:
                    Ebooks.ApplySearch(keyword);
                    break;
                case 2:
                    Bookmarks.ApplySearch(keyword);
                    break;
                case 3:
                    break;
                default:
                    Tabs.SelectedIndex = 0;
                    Papers.ApplySearch(keyword);
                    break;
            }
        }
    }
}
