var sciter;
var sys;

(async () => {
  sciter = await import("@sciter");
  sys = await import("@sys");

  Element.prototype.load = function(file) {
    this.content(sciter.decode(sys.fs.$readfile("dom/" + file)));
    return true;
  };
})();

var output_variables_proxied = {
  "username":"Not Set",
  "launcher_version":"0",
  "game_version":"Not Installed",
  "hash_progress":"0",
  "download_progress":"0",
  "patch_progress":"0",
  "update_progress":"0",
  "server_filter_players_min":0,
  "server_filter_players_max":64,
  "server_filter_same_game_version":true,
  "current_action": "None",
  "update_available": false,
};

var server_list_proxied = [];
var filtered_server_list = [];

const server_observer = { 
  set: function(target, prop, receiver) {
    console.log("server_observer");
    console.log("target: " + target);
    console.log("prop: " + prop);
    console.log("receiver: " + receiver);

    if (changeDefinition[0] == "update-range" || changeDefinition[0] == "add-range") {
      for (var value in receiver) {
        console.log("Should observe: " + value);
        //Object.addObserver(value, server_observer);
      }
      for(var i = filtered_server_list.length; i >= 0; i--) filtered_server_list.remove(i);
      var list = server_list.filter(server => server.display);
      for(var i = 0; i < list.length; i++) filtered_server_list.push(list[i]);
      console.log("Jsonified filtered_server_list: " + JSON.stringify(filtered_server_list));
    }
    if (changeDefinition[0] == "update" && prop == "display") {
      if(changeDefinition[3]) filtered_server_list.push(receiver);
      if(!changeDefinition[3]) filtered_server_list.removeByValue(receiver);
    }
    return Reflect.set(...arguments);
  }
}

const server_list = new Proxy(server_list_proxied, server_observer);


var news_items = [];
var footer;
var frame_id = 0;

function footer_progress() {
  footer = this;
  set_footer();
}

function set_footer() {
  switch(output_variables["current_action"]) {
    case "None":
      if(!output_variables["update_available"]) {
        footer.content("<div.hexpand.hflow.vcenter><p.uppercase.green.hexpand.vcenter>&#10003; Your game is up-to-date!</p><button.green #launch onclick=\"launchGame('');\">Launch to Menu</button></div>");
      } else {
        footer.content("<div.hexpand.hflow.vcenter><p.uppercase.red.hexpand.vcenter>&#10005; "+output_variables["popup_title"]+"</p><button.green #update onclick=\""+output_variables["button_onclick"]+"\">"+output_variables["popup_green"]+" Game</button></div>");
      }
      break;
    default:
      footer.content("<div.downloadBar><progressbar.indicator update_progress/></div><p.nowrap style=\"float:left;\"><output current_action/>: <span.green><output update_progress/>%</span></p><p overlay=\"verify.htm\" style=\"float:right;\">more details</p>");
      console.log("set_footer: Unhandled case for: " + output_variables["current_action"]);
  }
}

function news_feed_callback(text) {
  var arr = text.match(/(<item>(?:.|\n)+?<\/item>)/gm);
  var frame = document.$("frame");
  for (var i=0; i < arr.length; i++) {
    var text = arr[i];
    var item = {
      title: text.match(/<title>(?:<!\[CDATA\[)?([\w\W\_\-]+?)(?:\]\]>)?<\/title>/m)[1],
      link: text.match(/<link>(?:<!\[CDATA\[)?((?:.|\n)+?)(?:\?.+?)(?:\]\]>)?<\/link>/m)[1],
      description: text.match(/<description>(?:<!\[CDATA\[)?((?:.|\n)+?)(?:\]\]>)?<\/description>|<description\/()>/m)[1],
      pubDate: text.match(/<pubDate>(?:<!\[CDATA\[)?((?:.|\n)+?)(?:\]\]>)?<\/pubDate>/m)[1],
    };
    news_items.push(item);
    if (i==0) Window.this.xcall("fetch_resource", news_items[i].link+"?preview=1", {"Referer": "https://renegade-x.com/forums/forum/7-news/", "X-Requested-With": "XMLHttpRequest", "TE": "Trailers", "Pragma": "no-cache"}, load_news_item, {"id": i, "frame": undefined});
  }
}

function load_news_feed() {
  Window.this.xcall("fetch_resource", "https://renegade-x.com/rss/1-recent-news.xml/", {"Referer": "https://renegade-x.com/forums/forum/7-news/", "X-Requested-With": "XMLHttpRequest", "TE": "Trailers", "Pragma": "no-cache"}, news_feed_callback, this);
}

function image_callback(bytes) {
  var escaped_url = url.replace(/([\?\.\|\/\?\(\)])/g, "\\$1").trim();
  if(bytes) {
    var image = Image.fromBytes(bytes);
    if(image && url) {
      var url_regex = new RegExp(escaped_url, "g");
      var filetype = escaped_url.split('.').pop();
      news_items[id].content(news_items[id].html.replace(url_regex, "data:image/webp;base64,"+image.toBytes("#webp", 100).toString("base64")));
    } else {
      console.log("Image at url \""+escaped_url+"\" appears to be damaged.");
      var escaped_tag = "<img[^>]+?src=\""+escaped_url+"\"[^>]*?\/>";
      var image_regex = new RegExp(escaped_tag, "g");
      news_items[id].content(news_items[id].html.replace(image_regex, ""));
    }
  } else {
    console.log("Image at url \""+escaped_url+"\" appears to be missing.");
    var escaped_tag = "<img[^>]+?src=\""+escaped_url+"\"[^>]*?\/>";
    var image_regex = new RegExp(escaped_tag, "g");
    news_items[id].content(news_items[id].html.replace(image_regex, ""));
  }
  var regex = /<img[^>]+?src="(http[^"]+?\.(?!gif)[^"]{3,4}(?:\?[^"]+?)?)"[^>]*?>/;
  var img = news_items[id].html.match(regex);
  if (img && img[1]) Window.this.xcall("fetch_image", img[1], {}, image_callback, {id:id,url:img[1],frame:this.frame});
  else if (id==frame_id) {
    var news_frame = document.$("#news");
    if (this.frame) {
      this.frame.load(news_items[id].html, "");
    } else if (news_frame) {
      news_frame.load(news_items[id].html, "");
    }
  }
}

function load_news_item(text) {
  text = text.replace(/\s<\/span/g, "&nbsp;</span").replace(/[\r\n\s\t]+/g, " ").replace(/>\s</, ">&nbsp;<");
  var topicID = text.match(/data-topicID='(.+?)'/)[1];
  text = text.replace(new RegExp("_"+topicID,"g"), "_topicID");
  var youtube_regex = /<i?frame[^>]+?(?:src="(?:https?:\/\/)?(?:www\.|m\.)?(?:youtube\.com|youtu.be)\/(?:[\w\-\_]+\?v=|embed\/|v\/)?([\w\-\_]+)(?:\S+)?"[^>]*?)(?:\/>|>[^<>]*?<\/i?frame>)/g;
  text = text.replace(youtube_regex, "<a.playable href=\"https://youtube.com/watch?v=$1\"><img src=\"https://img.youtube.com/vi_webp/$1/maxresdefault.webp\"/></a>");

  var iframe_regex = /<i?frame[^>]*?(?:\/>|>[^<>]*?<\/i?frame>)/g;
  text = text.replace(iframe_regex, "");

  news_items[id].content(text);
  if (this.frame) this.frame.load(text,"");
  var regex = /<img[^>]+?src="(http[^"]+?\.(?!gif)[^"]{3,4}(?:\?[^"]+?)?)"[^>]*?>/;
  var img = text.match(regex);
  if (img && img[1]) Window.this.xcall("fetch_image", img[1], {}, image_callback, {id:id,url:img[1],frame: this.frame});
}

const variable_observer = { 
  set: function(target, prop, receiver) {
    for(const element of document.$$("output["+prop+"]") ) {
      element.setAttribute("value", Number(receiver)==0?"0":receiver);
    }
    for(const element of document.$$("progressbar["+prop+"]") ) {
      element.setAttribute("width", receiver==0?"0%":receiver+"%");
    }
    if(prop == "current_action" || prop == "update_available") {
      set_footer();
    }
    return Reflect.set(...arguments);
  }
}

const output_variables = new Proxy(output_variables_proxied, variable_observer);

function set_username(username) {
  output_variables["username"] = username;
  Window.this.xcall("set_playername", username);
}

function show_overlay(page) {
  if (document.$("div.menuEntries > .current")) {
    document.$("div.menuEntries > .current").classList.remove("current");
  }
  var overlay = document.$("#overlay");
  overlay.load(page);
  overlay.style["visibility"] = "visible";
  document.$("div.menuEntries").state.disabled = true;
}

function initialize_variables() {
  output_variables["username"] = Window.this.xcall("get_playername");
  output_variables["launcher_version"] = Window.this.xcall("get_launcher_version");
  output_variables["game_version"] = Window.this.xcall("get_game_version");
}

function onPingResult(server, time_response) {
  for (const s of server_list) {
    if ( s.data["IP"] + ":" + s.data["Port"] == server ) {
      s.latency = time_response + " ms";
      break;
    }
  }
}

function tick_checkmark(element, boolean) {
  if(boolean && !element.classList.contains("checked") && !element.classList.contains("ticked")) {
    element.classList.add("checked");
  } else if (!boolean && element.classList.contains("checked")) {
    element.classList.remove("checked");
  }
}

/// Joining game related things
function onGameExit() {
  var video = document.$("#map_video");
  if(video != undefined) {
    video.videoPlay();
  }
  console.log("Game exited succesfully!");
}

function onGameError(ErrorMessage) {
  var video = document.$("#map_video");
  if(video != undefined) {
    video.videoPlay();
  }
  show_overlay("error.htm");
  output_variables["error_message"] = ErrorMessage
}

function joinServer(password = undefined) {
  var servers = document.$("table.servers");
  var entry = servers.value[servers.tbody.currentIndex]["data"];
  if (entry["Variables"]["bPassworded"] && !password) {
    show_overlay("password.htm");
  } else {
    Window.this.xcall("launch_game",  entry["IP"]+":"+entry["Port"] + ( password? "?Password="+password : "" ), onGameExit, onGameError);
    var video = document.$("#map_video");
    if(video != undefined) {
      video.videoStop();
    }
  }
}

function launchGame(server, password = undefined) {
  Window.this.xcall("launch_game", server + ( password? "?Password="+password : "" ), onGameExit, onGameError);
  var video = document.$("#map_video");
  if(video != undefined) {
    video.videoStop();
  }
}

function getServersCallback(results) {
/* Example entry of results
  {
    "Name": "blabla",
    "Current Map": "CNC-LakeSide",
    "Bots": 1,
    "Players": 0,
    "Game Version": "5.48.145",
    "Variables": {
      "Mine Limit": 24,
      "bSteamRequired": false,
      "bPrivateMessageTeamOnly": false,
      "bPassworded": false,
      "bAllowPrivateMessaging": true,
      "bRanked": true,
      "Game Type": 1,
      "Player Limit": 10,
      "Vehicle Limit": 11,
      "bAutoBalanceTeams": false,
      "Team Mode": 6,
      "bSpawnCrates": true,
      "CrateRespawnAfterPickup": 35.0,
      "Time Limit": 0
    },
    "Port": 7777,
    "IP": "00.00.00.143"
  },
*/
  var updated = [];
  var old_length = server_list.length;
  output_variables["players_online"] = 0;
  for (var i = 0; i < old_length; i++) updated[i] = false;
  for(var changed in results) {
    if(!changed) continue;
    var in_range = changed["Players"] >= output_variables["server_filter_players_min"] &&  changed["Players"] <= output_variables["server_filter_players_max"];
    var result = {
      locked: changed["Variables"]["bPassworded"],
      name: changed["Name"],
      map: changed["Current Map"].substr(changed["Current Map"].indexOf("-")+1).replace("_", " "),
      players: changed["Players"] + "/" + changed["Variables"]["Player Limit"],
      latency: "-",

      display: in_range && changed["Game Version"] == output_variables["game_version"],
      in_player_range: in_range,
      same_version: changed["Game Version"] == output_variables["game_version"],
      data: changed,
    }
    output_variables["players_online"] += changed["Players"].toInteger();
    Window.this.xcall("get_ping", result.data["IP"]+":"+result.data["Port"], onPingResult);
    var exists = false;
    for (var i = 0; i < old_length; i++) {
      if (result.data.IP == server_list[i].data.IP && result.data.Port == server_list[i].data.Port) {
        exists = true;
        server_list[i] = result;
        updated[i] = true;
        break;
      }
    }
    if(!exists) {
      server_list.push(result);
    }
  }
  for (var i = old_length-1; i > 0; i--) if( !updated[i] ) server_list.remove(i);
  console.log("refreshed servers");
}

function updateFilter(arg1, arg2 = undefined) {
  if (arg2) {
    var min = arg1;
    var max = arg2;
    for(var i = 0; i < server_list.length; i++) {
      if(server_list[i].data["Players"] >= min && server_list[i].data["Players"] <= max) {
        server_list[i].in_player_range = true;
        console.log(server_list[i].data["Game Version"]);
        console.log(output_variables["game_version"]);
        server_list[i].display = document.$("div.filterbar > checkmark").classList.contains("checked")?(server_list[i].data["Game Version"] == output_variables["game_version"]):true;
      } else {
        server_list[i].in_player_range = false;
        server_list[i].display = false;
      }
    }
  } else {
    var same_version = arg1;
    for(var i = 0; i < server_list.length; i++) {
      server_list[i].display = server_list[i].in_player_range && same_version?(server_list[i].data["Game Version"] == output_variables["game_version"]):true;
    }
  }
}

function launcher_progress(progress) {
  var values = JSON.parse(progress);
  output_variables["download_progress"] = values[0] * 100 / values[1];
}

function update_launcher() {
  Window.this.xcall("update_launcher", launcher_progress);
  output_variables["current_action"] = "Installing launcher update";
  show_overlay("launcher-update.htm");
}

function check_launcher_result(new_version = undefined) {
  if(new_version != null) {
    console.log("New launcher version available: " + new_version);
    output_variables["popup_title"] = "A new launcher update is available";
    output_variables["popup_message"] = "Version " + new_version + " of the launcher is now available!";
    output_variables["popup_green"] = "UPDATE";
    show_overlay("popup_ok.htm");
    document.$("#overlay button.green").setAttribute("onclick", "update_launcher();")
    document.$("#overlay .close").setAttribute("onclick", "Window.this.xcall('check_update', onUpdateCallback, onUpdateErr);")
  } else {
    console.log("No new launcher version available.");
    Window.this.xcall("check_update", onUpdateCallback, onUpdateErr);
  }
}

function onProgress(progress) {
  output_variables["hash_progress_done"] = progress["hash"][0];
  output_variables["hash_progress_total"] = progress["hash"][1];
  if( progress["download"][1] != 0 && progress["hash"][1] == 0 ) {
    output_variables["hash_progress"] = 100;
  } else {
    output_variables["hash_progress"] = ( progress["hash"][1] != 0 )? progress["hash"][0]*100 / progress["hash"][1]:0;
  }

  output_variables["download_progress_done"] = progress["download"][0];
  output_variables["download_progress_total"] = progress["download"][1];
  var download_progress = ( progress["download"][1] != 0 )? progress["download"][0] * 100 / progress["download"][1] : 0.0;
  output_variables["download_progress"] = String.printf("%.1f", download_progress);
  output_variables["download_speed"] = progress["download_speed"];

  output_variables["patch_progress_done"] = progress["patch"][0];
  output_variables["patch_progress_total"] = progress["patch"][1];
  output_variables["patch_progress"] = ( progress["patch"][1] != 0 )? progress["patch"][0]*100 / progress["patch"][1]:0;
  output_variables["update_progress"] = String.printf("%.1f",(output_variables["hash_progress"]/3.0 + download_progress/3.0 + output_variables["patch_progress"]/3.0));
}

function onUpdateCallback(reason) {
  switch (reason) {
    case "up_to_date":
      break;
    case "full":
      output_variables["update_available"] = true;
      output_variables["popup_title"] = "The game is not installed";
      output_variables["popup_message"] = "Would you like to install?";
      output_variables["popup_green"] = "INSTALL";
      output_variables["popup_gray"] = "NOT NOW";
      output_variables["button_onclick"] = "Window.this.xcall(\"install_redists\", onRedistDone, onUpdateErr); output_variables[\"current_action\"] = \"Installing game dependencies\"; show_overlay(\"launcher-update.htm\");";
      show_overlay("popup_choice.htm");
      document.$("#overlay button.green").setAttribute("onclick", output_variables["button_onclick"]);
      break;
    case "resume":
      output_variables["current_action"] = "Resuming game installation";
      show_overlay("verify.htm");
      Window.this.xcall("start_download", onProgress, onUpdateDone, onUpdateErr);
      break;
    case "patch":
      output_variables["update_available"] = true;
      output_variables["popup_title"] = "A new game update is available";
      output_variables["popup_message"] = "Would you like to update?";
      output_variables["popup_green"] = "UPDATE";
      output_variables["popup_gray"] = "DELAY";
      output_variables["button_onclick"] = "Window.this.xcall(\"start_download\", onProgress, onUpdateDone, onUpdateErr); output_variables[\"current_action\"] = \"Updating game\"; show_overlay(\"verify.htm\");";
      show_overlay("popup_choice.htm");
      document.$("#overlay button.green").setAttribute("onclick", output_variables["button_onclick"]);
      break;
    case "validate":
      output_variables["current_action"] = "Validating game installation";
      show_overlay("verify.htm");
      Window.this.xcall("start_download", onProgress, onUpdateDone, onUpdateErr);
      break;
  }
}

function onUpdateErr(err) {
  var current_action = output_variables["current_action"];
  output_variables["current_action"] = "None";
  output_variables["popup_title"] = "An error has occurred while updating";
  output_variables["popup_message"] = "An error has occurred while \""+current_action+"\", the error states: </p><p>" + err;
  output_variables["popup_green"] = "Oh well...";
  show_overlay("popup_ok.htm");
}

function onRedistDone() {
  Window.this.xcall("start_download", onProgress, onUpdateDone, onUpdateErr);
  output_variables["current_action"] = "Installing game";
  if(document.$("#overlay") && document.$("#overlay").style["visibility"] == "visible") show_overlay("verify.htm");
}

function onUpdateDone() {
  var current_action = output_variables["current_action"];
  output_variables["current_action"] = "None";
  output_variables["update_available"] = false;
  output_variables["game_version"] = Window.this.xcall("get_game_version");
  Window.this.xcall("get_servers", getServersCallback);
  output_variables["popup_title"] = "Finished!";
  output_variables["popup_message"] = "The action \""+current_action+"\" was completed succesfully!";
  output_variables["popup_green"] = "OK";
  show_overlay("popup_ok.htm");
}

function onError(err) {
  var current_action = output_variables["current_action"];
  output_variables["current_action"] = "None";
  output_variables["popup_title"] = "An error has occurred";
  output_variables["popup_message"] = "An error has occurred during \""+current_action+"\", the error states: </p><p>" + err;
  output_variables["popup_green"] = "Oh well...";
  show_overlay("popup_ok.htm");
}

function resetGameUI() {
  output_variables["popup_title"] = "Reset game?";
  output_variables["popup_message"] = "This will reset your game to a newly installed state, removing any custom content that you may have, please confirm.";
  output_variables["popup_green"] = "I AM SURE";
  output_variables["popup_gray"] = "CANCEL";
  show_overlay("popup_choice.htm");
  document.$("#overlay button.green").setAttribute("onclick", "Window.this.xcall(\"remove_unversioned\", onUpdateCallback, onError); output_variables[\"current_action\"] = \"Removing unversioned files\"; show_overlay(\"verify.htm\"); ")
}