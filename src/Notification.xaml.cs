using System;
using System.IO;
using System.Windows;
using System.Media;
using System.Windows.Media.Imaging;
using System.Windows.Threading;

public partial class Notification : Window {
 public Notification(string message, string? title = null, string? imageUrl = null, string? soundFile = null, int timeout = 20) {
  InitializeComponent();
  if (!string.IsNullOrEmpty(title)) {
   TitleText.Text = title;
   TitleText.Visibility = Visibility.Visible;
  }
  if (!string.IsNullOrEmpty(imageUrl)) {
   try {
    BitmapImage bitmap = new BitmapImage();
    bitmap.BeginInit();
    bitmap.UriSource = new Uri(imageUrl);
    bitmap.EndInit();
    Image.Source = bitmap;
    Image.Visibility = Visibility.Visible;
   } catch (Exception ex) {
    Console.WriteLine("Cannot load image: " + ex.Message);
   }
  }
  DescriptionText.Text = message;
  var workingArea = System.Windows.SystemParameters.WorkArea;
  this.Left = workingArea.Right - this.Width - 10;
  this.Top = workingArea.Bottom - this.Height - 10;
  if (!string.IsNullOrEmpty(soundFile)) PlaySound(soundFile);
  var timer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(timeout) };
  timer.Tick += (sender, args) => {
   timer.Stop();
   this.Close();
  };
  timer.Start();
 }

 private void PlaySound(string soundFile) {
  try {
   SoundPlayer player = new SoundPlayer(soundFile);
   string soundFilePath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, soundFile);
   if (!System.IO.File.Exists(soundFilePath)) {
    Console.WriteLine("Sound file not found: " + soundFilePath);
    return;
   }
   player.Play();
  } catch (Exception ex) {
   Console.WriteLine("Cannot play notification sound: " + ex.Message);
  }
 }

 private void Close_Click(object sender, RoutedEventArgs e) {
  this.Close();
 }
}
