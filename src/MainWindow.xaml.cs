using System.Windows;
using System.Windows.Controls;
using Microsoft.Web.WebView2.Core;

/// <summary>
/// Interaction logic for MainWindow.xaml
/// </summary>
public partial class MainWindow : Window {
 public MainWindow() {
  InitializeComponent();
  InitializeAsync();
 }

 async void InitializeAsync() {
  await webView.EnsureCoreWebView2Async(null);
  webView.Source = new Uri("http://localhost:28282/");
  webView.CoreWebView2.WebMessageReceived += WebView_WebMessageReceived;
 }

 private void WebView_WebMessageReceived(object? sender, CoreWebView2WebMessageReceivedEventArgs e) {
  string message = e.TryGetWebMessageAsString();
  NotificationManager.ShowNotifications(message, "John Smith", "https://i.pravatar.cc/300?u=26", "message.wav", 20);
 }
 
 private void Window_Closing(object sender, System.ComponentModel.CancelEventArgs e) {
  Application.Current.Shutdown();
 }
}