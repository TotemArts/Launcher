  function loadOutput() {
    for(var name in this.attributes) {
      var attribute = output_variables[name];
      if(attribute==0) attribute = "0";
      if(attribute) {
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
  }

  function videoHandler() {
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
  }

  function news_image() {
    if (this.attributes.exists("width")) {
      this.attributes["width"] = (this.attributes["width"].toNumber()/10) + "%";
    } else {
      this.attributes["width"] = "100%";
    }
  }

  function chat_menu() {
    var chat = document.$("div.chat");
    stdout.println("Context menu enabled!");
    if (chat.selection.html != "") {
      stdout.println("Text selected: " + chat.selection.html);
    } else {
      this.$("#copy").state.disabled = true;
    }
  }

  function render_news_items() {
    var frame = document.$("#news")
    for (var i=0; i<news_items.length;i++) {
      var date = new Date(news_items[i].pubDate);
      var date_string = "<day>" + (date.day<10?'0':'') + date.day + "</day><month>" + date.monthName(false) + "</month>";
      var type_string = "General";
      if (news_items[i].title.match(/\sPATCH\s/i)) type_string = "Patch";
      this.append("<div.news_item.hflow id="+i+"><pubDate>"+date_string+"</pubDate><div.vflow><p.news_type>"+type_string+"</p><p.news_title>"+news_items[i].title+"</p></div></div>");
      var element = this.lastNode;
      element.on("click", function() {
        var id = this.attributes["id"].toNumber();
        frame_id = id;
        output_variables["current_news_title"] = news_items[id].title;
        var current = this.parent.$(".current");
        if (current) current.attributes.removeClass("current");
        this.attributes.addClass("current");
        if (news_items[id].html) {
          frame.load(news_items[id].html, "");
        } else {
          frame.load("", "");
          view.fetch_resource(news_items[id].link+"?preview=1", { "Referer": "https://renegade-x.com/forums/forum/7-news/", "X-Requested-With": "XMLHttpRequest", "TE": "Trailers", "Pragma": "no-cache"}, load_news_item, {id: id, frame: frame});
        }
      });
    }
    if (news_items.length > 0) {
      var id = 0;
      frame_id = 0;
      this.first.attributes.addClass("current");
      output_variables["current_news_title"] = news_items[0].title;
      if (news_items[0].html) {
        frame.load(news_items[0].html, "");
      } else {
        frame.load("", "");
        view.fetch_resource(news_items[id].link+"?preview=1", {Referer: "https://renegade-x.com/forums/forum/7-news/", "X-Requested-With": "XMLHttpRequest", TE: "Trailers", Pragma: "no-cache"}, load_news_item, {id: id, frame: frame});
      }
    }
  }

  function spoiler() {
    var spoiler = this.next;
    this.on("click", function() {
      if (spoiler.style["visibility"] == "collapse") {
        spoiler.style["visibility"] = "visible";
      } else if (spoiler.style["visibility"] == "visible") {
        spoiler.style["visibility"] = "collapse";
      } else {
        stdout.println("Weird");
      }
    });
  }

  function server_table() {
    this.value = filtered_server_list;

    this.tbody.currentIndex = 0;
    // The following event happens when the user changes the entry in the list, and will update the currently selected entry on the rest of the page
    this.on("change", function() {
        var entry = this.value[this.tbody.currentIndex].data;
        output_variables["title_menu"] = entry["Name"];
        document.$("#mine-limit").html = entry["Variables"]["Mine Limit"].toString();
        document.$("#player-limit").html = entry["Variables"]["Player Limit"].toString();
        document.$("#vehicle-limit").html = entry["Variables"]["Vehicle Limit"].toString();
        document.$("#time-limit").html = entry["Variables"]["Time Limit"].toString();
        tick_checkmark(document.$("checkmark#crates"), entry["Variables"]["bSpawnCrates"]);
        tick_checkmark(document.$("checkmark#steam"), entry["Variables"]["bSteamRequired"]);
        tick_checkmark(document.$("checkmark#ranked"), true);
        tick_checkmark(document.$("checkmark#balance"), entry["Variables"]["bAutoBalanceTeams"]);
        tick_checkmark(document.$("checkmark#infantry"), false);
        var currentMap = entry["Current Map"];
        var video = document.$("#map_video");
        video.videoLoad(view.get_video_location(entry["Current Map"]).replace("file:///", ""));
        video.videoPlay(0.0);
        var mapName = currentMap.split("-",1);
        document.$("#game-mode").html = mapName[0];
        document.$("#map-name").html = mapName[1].replace("_", " ");
      });
    this.on("click", "th.sortable", function() {
      this.sortVlist();
    });
    this.on("dblclick", "tr", function() {
      joinServer();
    });
  }

  function moveSliders() {
    var mousepressed = false;
    var element = this.$(".start");
    var min = this.attributes["minValue"].toInteger();
    var max = this.attributes["maxValue"].toInteger();
    var minPercentage = 100.0*this.attributes["min"].toFloat()/(max-min).toFloat();
    var maxPercentage = 100.0*this.attributes["max"].toFloat()/(max-min).toFloat();
    function updateRange() {
      this.$("div.slider > div.range").style["width"] = maxPercentage - minPercentage + "%";
      this.$("div.slider > div.range").style["left"] = minPercentage + "%";
      this.$("div.slider > div.range").style["right"] = "auto";
    }

    function updateElementByValue(integerValue) {
        var width_element = element.box("#width","#outer");
        var percentage_offset = 100.0*(width_element/2).toFloat()/element.parent.box("#width","#inner","#parent").toFloat();
        var snapToEvery = 100.0/(max - min).toFloat();
        element.style["left"] = integerValue.toFloat()*snapToEvery-percentage_offset+"%";
        element.style["right"] = "auto";
        if(element == this.$(".start")) {
          if(element.parent.attributes["min"] != min + integerValue) {
            element.parent.attributes["min"] = min + integerValue;
            minPercentage = integerValue.toFloat()*snapToEvery-percentage_offset;
            updateRange();
            element.parent.sendEvent(Event.CHANGE);
          }
        } else {
          if(element.parent.attributes["max"] != min + integerValue) {
            element.parent.attributes["max"] = min + integerValue;
            maxPercentage = integerValue.toFloat()*snapToEvery-percentage_offset;
            updateRange();
            element.parent.sendEvent(Event.CHANGE);
          }
        }
    }

    document.$("body").on("mousemove", function(evt) {
      if(mousepressed) {
        var left = element.parent.box("#left","#outer","#parent");
        var percentage = 100.0*(evt.x - left).toFloat()/element.parent.box("#width","#inner","#body").toFloat();
        var snapToEvery = 100.0/(max - min).toFloat();
        if(percentage > 100) percentage = 100.0;
        if(percentage < 0) percentage = 0.0;
        var integerValue = (percentage/snapToEvery).toInteger();
        if(element == element.parent.$(".start")) {
          if(integerValue + 1 >= element.parent.attributes["max"].toInteger()) integerValue = element.parent.attributes["max"].toInteger() - 1;
        } else {
          if(integerValue - 1 <= element.parent.attributes["min"].toInteger()) integerValue = element.parent.attributes["min"].toInteger() + 1;
        }
        updateElementByValue(integerValue);
      }
    });
    document.$("body").on("mouseup", function(evt) {
      mousepressed = false;
    });
    this.$(".end").on("mousedown", function(evt) {
      mousepressed = true;
      element = this;
    });
    this.$(".start").on("mousedown", function(evt) {
      mousepressed = true;
      element = this;
    });
    this.on("change", function(evt) {
      updateFilter(element.parent.attributes["min"].toInteger(), element.parent.attributes["max"].toInteger());
    });
  }


function bool_setting() {
  this.post(this.attributes.addClass(view.get_setting(this.getAttribute("setting"))));

  this.on("click", function(evt) {
    if(this.attributes.hasClass("true")) {
      this.attributes.removeClass("true");
      this.attributes.addClass("false");
      view.set_setting(this.getAttribute("setting"), "false");
    } else if (this.attributes.hasClass("false")) {
      this.attributes.removeClass("false");
      this.attributes.addClass("true");
      view.set_setting(this.getAttribute("setting"), "true");
    }
  });
}

function filter() {
  var filterbar = document.$(".filterbar");

  this.on("click", function(evt) {
    if(this.attributes.hasClass("down")) {
      this.attributes.removeClass("down");
      this.attributes.addClass("up");
      filterbar.style["visibility"] = "visible";
    } else if (this.attributes.hasClass("up")) {
      this.attributes.removeClass("up");
      this.attributes.addClass("down");
      filterbar.style["visibility"] = "collapse";
    }
  });
}

document.on("keydown", function(evt) {
  if ( evt.keyCode == Event.VK_F5 ) {
    self.reload();
  }
});

document.on("~click", "a[href^=http]", function(evt) {
  var url = evt.target.attributes["href"];
  Sciter.launch(url);
  return true;
});

document.on("~click", "checkmark[toggle]", function(evt) {
  if (!evt.target.attributes.hasClass("checked")) {
    evt.target.attributes.addClass("checked");
    updateFilter(true);
  } else {
    evt.target.attributes.removeClass("checked");
    updateFilter(false);
  }
  return true;
});

function reload() {
  if( this.parent ) this.parent.load( this.url() );
  else view.load(this.url());
}

function fillHeight() {
  this.onSize = function() {
    var min_width = 0;
    for (var child in this) {
      min_width += child.toPixels(child.style["-min"]);
    }
    var parent_width = this.box("#width", "#border", "#parent");
    for (var child in this) {
      if( parent_width >= min_width ) {
        if(this.style["flow"] != "horizontal") {
          this.style["flow"] = "horizontal";
        }
      } else {
        if(this.style["flow"] != "vertical") {
          this.style["flow"] = "vertical";
        }
      }
    }
  };
  this.onSize();
}

self.on("click","[onclick]",function() {
  eval.call(this, this.attributes["onclick"] );
  return false;
});

var current_page;

self.on("click","[page]",function() {
  document.$("div.menuEntries > .current").attributes.removeClass("current");
  this.attributes.addClass("current");
  current_page = this;
  document.$("#content").load(this.attributes["page"]);
  return false;
});

self.on("click","[overlay]",function() {
  document.$("div.menuEntries > .current").attributes.removeClass("current");
  this.attributes.addClass("current");
  var overlay = document.$("#overlay");
  overlay.load(this.attributes["overlay"]);
  overlay.style["visibility"] = "visible";
  document.$("div.menuEntries").state.disabled = true;
  return false;
});

self.on("click","[close]",function() {
  close_overlay();
  return false;
});

self.on("click","[external]",function() {
  stdout.println(this.attributes["external"]);
  Sciter.launch(this.attributes["external"]);
  return false;
});

self.on("keyup","[onkey]",function(evt) {
  eval.call(this, this.attributes["onkey"]);
});

self.on("keypress","[enter]",function(evt) {
  if ( evt.keyCode != 13 && evt.keyCode != Event.VK_RETURN ) return;
  eval.call(this, this.attributes["enter"]);
});