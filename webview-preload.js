const { ipcRenderer } = require("electron");

const OriginalNotification = window.Notification;

window.Notification = function (title, options) {
  const notification = new OriginalNotification(title, options);

  notification.addEventListener("click", () => {
    ipcRenderer.sendToHost("notification-clicked");
  });

  return notification;
};

window.Notification.permission = OriginalNotification.permission;
window.Notification.requestPermission = OriginalNotification.requestPermission;
