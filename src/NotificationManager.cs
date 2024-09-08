using System.Collections.Generic;
using System.Linq;
using System.Windows;

public class NotificationManager {
 private const int maxNotifications = 10;
 private static List<Notification> notificationList = new List<Notification>();

 public static void ShowNotifications(string message, string? title = null, string? imageUrl = null, string? soundFile = null, int timeout = 20) {
  if (notificationList.Count >= maxNotifications) {
   Notification oldestNotification = notificationList[0];
   oldestNotification.Close();
   notificationList.RemoveAt(0);
  }
  Notification notification = new Notification(message, title, imageUrl, soundFile, timeout);
  notification.Show();
  foreach (Notification n in notificationList) {
   n.Top -= notification.Height + 10;
  }
  notificationList.Add(notification);
  notification.Closed += (s, e) => {
   notificationList.Remove(notification);
   foreach (Notification n in notificationList) {
    if (n.Top < notification.Top) n.Top += notification.Height + 10;
   }
  };
 }
}
