Date.prototype.monthName = function(longFormat) {
  return this.toUTCString().split(' ')[2]
};

var sciter;
var sys;

(async () => {
  sciter = await import("@sciter");
  sys = await import("@sys");
})();

var Emu = {
  loadOutput: function() {
    console.log("loadOutput " + this);
    for(var name of this.getAttributeNames()) {
      console.log("Checking value for " + name);
      var attribute = output_variables[name];
      if(attribute==0) attribute = "0";
      if(attribute) {
        console.log("setting value for " + name);
        switch (this.tag) {
          case "output":
            this.value = attribute;
            break;
          case "progressbar":
            this.style["width"] = attribute + "%";
            break;
        }
      }
    }
  },

  videoHandler: function() {
    var video = this;
    video.shouldPlay = true;

    video.onControlEvent = function(evt) {
      switch(evt.type) {
        case Event.VIDEO_INITIALIZED:
          return false;
        case Event.VIDEO_STARTED:
          return false;
        case Event.VIDEO_STOPPED:
          if(this.videoIsEnded())
            video.videoPlay(0.0);
          return false; 
      }
    }  
  },

  news_image: function() {
    if (this.getAttribute("width")) {
      console.log(this.state.contentWidths());
      var minMax = this.state.contentWidths();
      if( minMax[1] > 750 ) {
        this.setAttribute("width", "750dip");
      }
    }
  },

  chat_menu: function() {
    var chat = document.$("div.chat");
    console.log("Context menu enabled!");
    if (chat.selection.html != "") {
      console.log("Text selected: " + chat.selection.html);
    } else {
      this.$("#copy").state.disabled = true;
    }
  },

  render_news_items: function() {
    var frame = document.$("#news")
    for (var i=0; i<news_items.length;i++) {
      var date = new Date(news_items[i].pubDate);
      var date_string = "<day>" + (date.getDay() <10?'0':'') + date.getDay() + "</day><month>" + date.monthName(false) + "</month>";
      var type_string = "General";
      if (news_items[i].title.match(/\sPATCH\s/i)) type_string = "Patch";
      this.append("<div.news_item.hflow id="+i+"><pubDate>"+date_string+"</pubDate><div.vflow><p.news_type>"+type_string+"</p><p.news_title>"+news_items[i].title+"</p></div></div>");
      var element = this.lastChild;
      element.on("click", function(evt) {
        var id = Number(evt.target.getAttribute("id"));
        frame_id = id;
        output_variables["current_news_title"] = news_items[id].title;
        var current = evt.target.parentElement.$(".current");
        if (current) current.classList.remove("current");
        evt.target.classList.add("current");
        if (news_items[id].html) {
          console.log("Setting the html to news_items["+id+"]");
          console.log(frame);
          console.log(frame.load);
          frame.load(news_items[id].html, "");
        } else {
          console.log("Clearing the frame and fetching resource");
          frame.load("", "");
          Window.this.xcall("fetch_resource", news_items[id].link+"?preview=1", { "Referer": "https://ren-x.com/forums/forum/7-news/", "X-Requested-With": "XMLHttpRequest", "TE": "Trailers", "Pragma": "no-cache"}, load_news_item, {id: id, frame: frame});
        }
      });
    }
    if (news_items.length > 0) {
      var id = 0;
      frame_id = 0;
      this.firstElementChild.classList.add("current");
      output_variables["current_news_title"] = news_items[0].title;
      if (news_items[0].html) {
        frame.load(news_items[0].html, "");
      } else {
        frame.load("", "");
        Window.this.xcall("fetch_resource", news_items[id].link+"?preview=1", {Referer: "https://ren-x.com/forums/forum/7-news/", "X-Requested-With": "XMLHttpRequest", TE: "Trailers", Pragma: "no-cache"}, load_news_item, {id: id, frame: frame});
      }
    }
  },

  spoiler: function() {

    var spoiler = this.nextElementSibling;
    this.on("click", function(evt) {
      if (spoiler.style["visibility"] == "collapse") {
        spoiler.style["visibility"] = "visible";
      } else if (spoiler.style["visibility"] == "visible") {
        spoiler.style["visibility"] = "collapse";
      } else {
        console.log("Weird");
      }
    });
  },

  server_table: function() {
    this.value = filtered_server_list;

    this.$("tbody").currentIndex = 0;
    // The following event happens when the user changes the entry in the list, and will update the currently selected entry on the rest of the page
    this.on("change", function(evt) {
        var entry = evt.target.value[evt.target.tbody.currentIndex].data;
        output_variables["title_menu"] = entry["Name"];
        document.$("#mine-limit").content(entry["Variables"]["Mine Limit"].toString());
        document.$("#player-limit").content(entry["Variables"]["Player Limit"].toString());
        document.$("#vehicle-limit").content(entry["Variables"]["Vehicle Limit"].toString());
        document.$("#time-limit").content(entry["Variables"]["Time Limit"].toString());
        tick_checkmark(document.$("checkmark#crates"), entry["Variables"]["bSpawnCrates"]);
        tick_checkmark(document.$("checkmark#steam"), entry["Variables"]["bSteamRequired"]);
        tick_checkmark(document.$("checkmark#ranked"), true);
        tick_checkmark(document.$("checkmark#balance"), entry["Variables"]["bAutoBalanceTeams"]);
        tick_checkmark(document.$("checkmark#infantry"), false);
        var currentMap = entry["Current Map"];
        var video = document.$("#map_video");
        video.videoLoad(Window.this.xcall("get_video_location", entry["Current Map"]).replace("file:///", ""));
        video.videoPlay(0.0);
        var mapName = currentMap.split("-",1);
        document.$("#game-mode").content(mapName[0]);
        document.$("#map-name").content(mapName[1].replace("_", " "));
      });
    this.on("click", "th.sortable", function(evt) {
      evt.target.sortVlist();
    });
    this.on("dblclick", "tr", function() {
      joinServer();
    });
  },

  moveSliders: function() {
    var mousepressed = false;
    var element = this.$(".start");
    var min = Number(this.getAttribute("minValue"));
    var max = Number(this.getAttribute("maxValue"));
    var minPercentage = 100.0*this.getAttribute("min")/(max-min);
    var maxPercentage = 100.0*this.getAttribute("max")/(max-min);
    function updateRange() {
      this.$("div.slider > div.range").style["width"] = maxPercentage - minPercentage + "%";
      this.$("div.slider > div.range").style["left"] = minPercentage + "%";
      this.$("div.slider > div.range").style["right"] = "auto";
    }

    function updateElementByValue(integerValue) {
        var width_element = element.state.box("width","outer");
        var percentage_offset = 100.0*(width_element/2)/element.parentElement.state.box("width","inner","parent");
        var snapToEvery = 100.0/(max - min);
        element.style["left"] = integerValue*snapToEvery-percentage_offset+"%";
        element.style["right"] = "auto";
        if(element == this.$(".start")) {
          if(element.parentElement.getAttribute("min") != min + integerValue) {
            element.parentElement.setAttribute("min", min + integerValue);
            minPercentage = integerValue*snapToEvery-percentage_offset;
            updateRange();
            element.parentElement.post(Event.CHANGE);
          }
        } else {
          if(element.parentElement.getAttribute("max") != min + integerValue) {
            element.parentElement.setAttribute("max", min + integerValue);
            maxPercentage = integerValue*snapToEvery-percentage_offset;
            updateRange();
            element.parentElement.post(Event.CHANGE);
          }
        }
    }

    document.$("body").on("mousemove", function(evt) {
      if(mousepressed) {
        var left = element.parentElement.state.box("left","outer","parent");
        var percentage = 100.0*(evt.x - left)/element.parentElement.state.box("width","inner","#body");
        var snapToEvery = 100.0/(max - min);
        if(percentage > 100) percentage = 100.0;
        if(percentage < 0) percentage = 0.0;
        var integerValue = (percentage/snapToEvery);
        if(element == element.parentElement.$(".start")) {
          if(integerValue + 1 >= Number(element.parentElement.getAttribute("max"))) integerValue = Number(element.parentElement.getAttribute("max")) - 1;
        } else {
          if(integerValue - 1 <= Number(element.parentElement.getAttribute("min"))) integerValue = Number(element.parentElement.getAttribute("min")) + 1;
        }
        updateElementByValue(integerValue);
      }
    });
    document.$("body").on("mouseup", function(evt) {
      mousepressed = false;
    });
    this.$(".end").on("mousedown", function(evt) {
      mousepressed = true;
      element = evt.target;
    });
    this.$(".start").on("mousedown", function(evt) {
      mousepressed = true;
      element = evt.target;
    });
    this.on("change", function(evt) {
      updateFilter(Number(element.parentElement.getAttribute("min")), Number(element.parentElement.getAttribute("max")));
    });
  }
}

function bool_setting() {
  this.classList.add(Window.this.xcall("get_setting", this.getAttribute("setting")));

  this.on("click", function(evt) {
    if(evt.target.classList.contains("true")) {
      evt.target.classList.remove("true");
      evt.target.classList.add("false");
      Window.this.xcall("set_setting", evt.target.getAttribute("setting"), "false");
    } else if (evt.target.classList.contains("false")) {
      evt.target.classList.remove("false");
      evt.target.classList.add("true");
      Window.this.xcall("set_setting", evt.target.getAttribute("setting"), "true");
    }
  });
}

function filter() {
  var filterbar = document.$(".filterbar");

  this.on("click", function(evt) {
    if(evt.target.classList.contains("down")) {
      evt.target.classList.remove("down");
      evt.target.classList.add("up");
      filterbar.style["visibility"] = "visible";
    } else if (evt.target.classList.contains("up")) {
      evt.target.classList.remove("up");
      evt.target.classList.add("down");
      filterbar.style["visibility"] = "collapse";
    }
  });
}

document.on("keydown", function(evt) {
  if ( evt.keyCode == Event.VK_F5 ) {
    document.reload();
  }
});

document.on("~click", "a[href^=http]", function(evt) {
  var url = evt.target.getAttribute("href");
  sciter.launch(url);
  return true;
});

document.on("~click", "checkmark[toggle]", function(evt) {
  if (!evt.target.classList.contains("checked")) {
    evt.target.classList.add("checked");
    updateFilter(true);
  } else {
    evt.target.classList.remove("checked");
    updateFilter(false);
  }
  return true;
});

function reload() {
  if( this.parent ) this.parentElement.load( this.url() );
  else Window.this.xcall("load", this.url());
}

function fillHeight() {
  this.onSize = function(evt) {
    var min_width = 0;
    for (var child of evt.target) {
      console.log(child);
      min_width += child.toPixels(child.style["-min"]);
    }
    var parent_width = evt.target.state.box("width", "border", "parent");
    for (var child of evt.target) {
      if( parent_width >= min_width ) {
        if(evt.target.style["flow"] != "horizontal") {
          evt.target.style["flow"] = "horizontal";
        }
      } else {
        if(evt.target.style["flow"] != "vertical") {
          evt.target.style["flow"] = "vertical";
        }
      }
    }
  };

}

document.on("click","[onclick]",function(evt) {
  console.log("Executing eval of: \""+evt.target.getAttribute("onclick")+"\"");
  eval.call(evt.target, evt.target.getAttribute("onclick") );
  return false;
});

var current_page;

function set_current_page(page) {
  current_page = page;
}

function close_overlay() {
  if (document.$("div.menuEntries > .current")) {
    document.$("div.menuEntries > .current").classList.remove("current");
  }
  current_page.classList.add("current");
  var overlay = document.$("#overlay");
  overlay.text = "";
  overlay.style["visibility"] = "collapse";
  document.$("div.menuEntries").state.disabled = false;
}

document.on("click","[page]",function(evt) {
  document.$("div.menuEntries > .current").classList.remove("current");
  evt.target.classList.add("current");
  current_page = evt.target;
  document.$("#content").load(evt.target.getAttribute("page"));
  return false;
});

document.on("click","[overlay]",function(evt) {
  document.$("div.menuEntries > .current").classList.remove("current");
  evt.target.classList.add("current");
  var overlay = document.$("#overlay");
  overlay.load(evt.target.getAttribute("overlay"));
  overlay.style["visibility"] = "visible";
  document.$("div.menuEntries").state.disabled = true;
  return false;
});

document.on("click","[close]",function() {
  close_overlay();
  return false;
});

document.on("keyup","[onkey]",function(evt) {
  eval.call(evt.target, evt.target.getAttribute("onkey"));
});

document.on("keypress","[enter]",function(evt) {
  if ( evt.keyCode != 13 && evt.keyCode != Event.VK_RETURN ) return;
  eval.call(evt.target, evt.target.getAttribute("enter"));
});