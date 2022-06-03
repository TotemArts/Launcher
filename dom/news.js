import { toBase64 } from "@sciter";

globalThis.news_feed_callback = function (text) {
  try {
    var arr = text.match(/(<item>(?:.|\n)+?<\/item>)/gm);
    globalThis.news_items = [];
    for (var i = 0; i < arr.length; i++) {
      var text = arr[i];
      var item = {
        id: i,
        title: text.match(/<title>(?:<!\[CDATA\[)?([\w\W\_\-]+?)(?:\]\]>)?<\/title>/m)[1],
        link: text.match(/<link>(?:<!\[CDATA\[)?((?:.|\n)+?)(?:\?.+?)(?:\]\]>)?<\/link>/m)[1],
        description: text.match(/<description>(?:<!\[CDATA\[)?((?:.|\n)+?)(?:\]\]>)?<\/description>|<description\/()>/m)[1],
        pubDate: text.match(/<pubDate>(?:<!\[CDATA\[)?((?:.|\n)+?)(?:\]\]>)?<\/pubDate>/m)[1],
      };
      globalThis.news_items.push(item);
      if (i == 0) Window.this.xcall("fetch_resource", item.link + "?preview=1", { "Referer": "https://totemarts.games/forums/forum/7-news/", "X-Requested-With": "XMLHttpRequest", "TE": "Trailers", "Pragma": "no-cache" }, globalThis.load_news_item, { "id": i });
    }
  } catch (e) {
    console.error("news_feed_callback");
    console.error(printf("Caught exception: %s\n%V", e, e.stacktrace));;
  }
}

globalThis.htmlToJsx = (html) => globalThis.objectToJsx(Window.this.xcall("html_to_jsx", html));

globalThis.objectToJsx = (object) => {
  if(!Array.isArray(object)) {
    return object;
  }
  if(object[1]["id"] == "ipsTabs_elTopic_topicID_elTopic_topicID_latestPost_panel") {
    return "";
  }
  if(object[1]["id"] == "elTopic_topicID") {
    return "";
  }
  if(object[1]["class"] != undefined && object[1]["class"].includes("ipsPhotoPanel")) {
    return "";
  }
  if(object[0] == "hr") {
    return "";
  }
    
  var children = [];
  for(var transform of object[2]) {
    children.push(globalThis.objectToJsx(transform));
  };
  return JSX(object[0], object[1], children);
};

globalThis.load_news_item = function (text) {
  try {
    // preserve spaces in render:
    text = text.replace(/[\r\n\s\t]+/g, " ");
    // get topic id:
    var topicID = text.match(/data-topicID='(.+?)'/)[1];
    text = text.replace(new RegExp("_" + topicID, "g"), "_topicID");
    // replace youtube videos with thumbnail links
    var youtube_regex = /<i?frame[^>]+?(?:src="(?:https?:\/\/)?(?:www\.|m\.)?(?:youtube\.com|youtu.be)\/(?:[\w\-\_]+\?v=|embed\/|v\/)?([\w\-\_]+)(?:\S+)?"[^>]*?)(?:\/>|>[^<>]*?<\/i?frame>)/g;
    text = text.replace(youtube_regex, "<a.playable href=\"https://youtube.com/watch?v=$1\"><img src=\"https://img.youtube.com/vi_webp/$1/maxresdefault.webp\"/></a>");

    // Disable iframe's
    var iframe_regex = /<i?frame[^>]*?(?:\/>|>[^<>]*?<\/i?frame>)/g;
    text = text.replace(iframe_regex, "External content hidden");

    var image_regex = /<img([^>]*?)\/?>/g;
    text = text.replace(image_regex, "<img$1/>");

    var horizontal_regex = /<hr([^>]*?)\/?>/g;
    text = text.replace(horizontal_regex, "<hr$1/>");

    var break_regex = /<br([^>]*?)\/?>/g;
    text = text.replace(break_regex, "<br$1/>");

    // set the html of news_items[this.id];
    globalThis.news_items[this.id].html = globalThis.htmlToJsx(text);

    globalThis.callback_service.publish("news", { id: this.id });

    var url = globalThis.getUntransformedImageUrl(globalThis.news_items[this.id].html);
    if (url != undefined) {
      Window.this.xcall("fetch_image", url, {}, globalThis.image_callback, { id: this.id, url: url });
    }
  } catch (e) {
    console.error("load_news_item");
    console.error(printf("Caught exception: %s\n%V", e, e.stacktrace));
  }
}

globalThis.replaceImageSource = function(source, from, to) {
  if(!Array.isArray(source)) return;
  if(source[0] == "img" && source[1]["src"] == from) {
    source[1]["src"] = to;
  }
  for(var child of source[2]) {
    globalThis.replaceImageSource(child, from, to);
  }
};
globalThis.removeImageWithUrl = function(source, url) {
  for(var child of source[2]) {
    if(!Array.isArray(child)) continue;
    if(child[0] == "img" && child[1]["src"] == url) {
      source[2].splice(source[2].indexOf(child),1);
    }
    globalThis.removeImageWithUrl(child, url);
  }
};
globalThis.getUntransformedImageUrl = function(source) {
  
  for(var child of source[2]) {
    if(!Array.isArray(child)) continue;
    if(child[0] == "img") {
      var img = child[1]["src"].match(/http.+\.(?!gif).{3,4}(?:\?.+)?/);
      if (img && img[0]) return child[1]["src"];
    }
    var url = globalThis.getUntransformedImageUrl(child);
    if(url != undefined) return url;
  }
  return undefined;
};

globalThis.image_callback = function (image) {
  try {
    if (image) {
      let bytes, base64;
      bytes = image.toBytes("webp", 100);
      base64 = toBase64(bytes);

      globalThis.replaceImageSource(globalThis.news_items[this.id].html, this.url, "data:image/webp;base64," + base64);
    } else {
      console.log("Image at url \"" + this.url + "\" appears to be missing.");
      globalThis.removeImageWithUrl(globalThis.news_items[this.id].html, this.url);
    }
    var url = globalThis.getUntransformedImageUrl(globalThis.news_items[this.id].html);
    if (url != undefined) {
      Window.this.xcall("fetch_image", url, {}, globalThis.image_callback, { id: this.id, url: url });
    }
  } catch (e) {
    console.error("image_callback");
    console.error(printf("Caught exception: %s\n%V", e, e.stacktrace));;
  }
}

export class News extends Element {
  constructor(props, kids) {
    super(props, kids);
  }

  current_news_id = 0;
  current_news = globalThis.news_items[this.current_news_id];
  body_width = globalThis.document.body.state.box("width", "border", "parent");

  render() {
    return <div class="news-grid">
      <div class="logo hflow">
        <div class="vflow vcenter">
          <p style="font-size: 7pt;">WELCOME</p>
          <h1 class="change" overlay="username">
            {globalThis.username}
          </h1>
        </div>
      </div>
      <div class="left-margin"></div>
      <div class="right-margin"></div>
      <div class="footer-margin"></div>
      <div class="hflow child-margin expand">
        {this.render_news()}
      </div>
    </div>
  }

  render_news() {
    var news_items = this.render_news_feed();

    if(this.body_width < 1290) {
      var list = [];
      for(const news_item of news_items) {
        list.push(news_item);
        if(news_item[1]["id"] == this.current_news_id) list.push(<div id="news">{this.current_news.html}</div>);
      }
      
      return <div class="child-margin expand" style="width: *;">
        <div class="news_items_container vflow" style="width: *;">
          <div class="titlebar">
            <h3 class="title uppercase">News</h3>
          </div>
          <div class="expand">
            {list}
          </div>
        </div>
      </div>;
    }

    var news_item = this.render_news_article();
    return <div class="hflow child-margin expand">
      <div class="news_items_container vflow">
        <div class="titlebar">
          <h3 class="title uppercase">News</h3>
        </div>
        <div class="expand">
          {news_items}
        </div>
      </div>
      {news_item}
    </div>;
  }

  render_news_feed() {
    var list = [];
    for (const item of globalThis.news_items) {
      var date = new Date(item.pubDate);
      var type_string = "General";
      if (item.title.match(/\sPATCH\s/i))
        type_string = "Patch";
      var classes = "news_item hflow";
      if (this.current_news_id == item.id)
        classes += " current"
      list.push(<div class={classes} id={item.id}>
        <pubDate><day> {(date.getDate() < 10 ? '0' : '') + date.getDate()}</day><month>{date.toUTCString().split(' ')[2]}</month></pubDate>
        <div class="vflow">
          <p class="news_type">{type_string}</p>
          <p class="news_title">{item.title}</p>
        </div>
      </div>);
    }
    return list;
  }

  render_news_article() {
    if (globalThis.news_items.length == 0)
      return <div></div>

    return <div class="news_container vflow">
      <div class="titlebar">
        <h3 class="title">{this.current_news.title}</h3>
      </div>
      <div>
        <div id="news">{this.current_news.html}</div>
      </div>
      <div id="news-footer">View the full thread: <a href={this.current_news.link} target="@system">{this.current_news.title}</a></div>
    </div>;
  }

  componentDidMount() {
    globalThis.callback_service.subscribe("news", this, this.callback);
    this.onsizechange = (evt, target) => {
      var width = globalThis.document.body.state.box("width", "border", "parent");
      this.componentUpdate({ body_width: width });
    };
  }

  callback(data) {
    if(data.id == this.current_news_id) {
      this.componentUpdate({ current_news: Object.assign({}, globalThis.news_items[data.id]) });
    }
  }

  componentWillUnmount() {
    globalThis.callback_service.unsubscribe("news", this, this.callback);
  }

  ["on click at .ipsSpoiler_header,.ipsQuote_citation"](evt, target) {
    var spoiler = target.nextElementSibling;
    if (spoiler.style["visibility"] == "collapse") {
      spoiler.style["visibility"] = "visible";
    } else if (spoiler.style["visibility"] == "visible") {
      spoiler.style["visibility"] = "collapse";
    } else {
      console.log("Weird");
    }
  }

  ["on click at div.news_item[id]"](evt, target) {
    var id = parseInt(target.getAttribute("id"));
    var item = globalThis.news_items[id];
    this.componentUpdate({ current_news_id: id, current_news: item});
    if (!item.html) {
      Window.this.xcall("fetch_resource", item.link + "?preview=1", { "Referer": "https://totemarts.games/forums/forum/7-news/", "X-Requested-With": "XMLHttpRequest", "TE": "Trailers", "Pragma": "no-cache" }, globalThis.load_news_item, { "id": id });
    }
  }
}