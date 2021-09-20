var messages = {};

function newMessages(id, message) {
  var shouldScroll = (document.$("div.chat").scroll("bottom") > 0);

  document.$("div.chat").append("<message id=\"" + id + "\">" + message + "</message>");

  if (!shouldScroll) {
    scrollToBottom();
  }
}

function scrollToBottom() {
  var chat = document.$("div.chat");
  chat.scrollTo(0, chat.scroll("bottom") + chat.scroll("#height") + chat.scroll("top"), true);
}


