import { toBase64 } from "@sciter";

globalThis.news_feed_callback = function(text) {
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
      if (i == 0) Window.this.xcall("fetch_resource", item.link + "?preview=1", { "Referer": "https://ren-x.com/forums/forum/7-news/", "X-Requested-With": "XMLHttpRequest", "TE": "Trailers", "Pragma": "no-cache" }, globalThis.load_news_item, { "id": i });
    }
  } catch (e) {
    console.error("news_feed_callback");
    console.error(printf("Caught exception: %s\n%V", e, e.stacktrace));;
  }
}

globalThis.load_news_item = function(text) {
  try {
    text = text.replace(/\s<\/span/g, "&nbsp;</span").replace(/[\r\n\s\t]+/g, " ").replace(/>\s</, ">&nbsp;<");
    var topicID = text.match(/data-topicID='(.+?)'/)[1];
    text = text.replace(new RegExp("_" + topicID, "g"), "_topicID");
    var youtube_regex = /<i?frame[^>]+?(?:src="(?:https?:\/\/)?(?:www\.|m\.)?(?:youtube\.com|youtu.be)\/(?:[\w\-\_]+\?v=|embed\/|v\/)?([\w\-\_]+)(?:\S+)?"[^>]*?)(?:\/>|>[^<>]*?<\/i?frame>)/g;
    text = text.replace(youtube_regex, "<a.playable href=\"https://youtube.com/watch?v=$1\"><img src=\"https://img.youtube.com/vi_webp/$1/maxresdefault.webp\"/></a>");

    var iframe_regex = /<i?frame[^>]*?(?:\/>|>[^<>]*?<\/i?frame>)/g;
    text = text.replace(iframe_regex, "");
    globalThis.news_items[this.id].html = text;

    var regex = /<img[^>]+?src="(http[^"]+?\.(?!gif)[^"]{3,4}(?:\?[^"]+?)?)"[^>]*?>/;
    var img = text.match(regex);
    if (img && img[1]) Window.this.xcall("fetch_image", img[1], {}, globalThis.image_callback, { id: this.id, url: img[1] });
  } catch (e) {
    console.error("load_news_item");
    console.error(printf("Caught exception: %s\n%V", e, e.stacktrace));;
  }
}

globalThis.image_callback = function(image2) {
  try {
    var escaped_url = this.url.replace(/([\?\.\|\/\?\(\)])/g, "\\$1").trim();
    if (image2) {
      let image = Graphics.Image.fromBytes(image2);
      if (image && this.url) {
        var url_regex = new RegExp(escaped_url, "g");
        let bytes, base64;
        bytes = image.toBytes("webp", 100);
        base64 = toBase64(bytes);
        globalThis.news_items[this.id].html = globalThis.news_items[this.id].html.replace(url_regex, "data:image/webp;base64," + base64);
      } else {
        console.log("Image at url \"" + escaped_url + "\" appears to be damaged.");
        var escaped_tag = "<img[^>]+?src=\"" + escaped_url + "\"[^>]*?\/>";
        var image_regex = new RegExp(escaped_tag, "g");
        globalThis.news_items[this.id].html = globalThis.news_items[this.id].html.replace(image_regex, "");
      }
    } else {
      console.log("Image at url \"" + escaped_url + "\" appears to be missing.");
      var escaped_tag = "<img[^>]+?src=\"" + escaped_url + "\"[^>]*?\/>";
      var image_regex = new RegExp(escaped_tag, "g");
      globalThis.news_items[this.id].html = globalThis.news_items[this.id].html.replace(image_regex, "");
    }
    var regex = /<img[^>]+?src="(http[^"]+?\.(?!gif)[^"]{3,4}(?:\?[^"]+?)?)"[^>]*?>/;
    var img = globalThis.news_items[this.id].html.match(regex);
    if (img && img[1]) Window.this.xcall("fetch_image", img[1], {}, globalThis.image_callback, { id: this.id, url: img[1] });
  } catch (e) {
    console.error("image_callback");
    console.error(printf("Caught exception: %s\n%V", e, e.stacktrace));;
  }
}

export class News extends Element {
  constructor(props, kids) {
    super(props, kids);
  }

  current_news_title = "";

  render() {
    var news_items = this.render_news_feed();
    var news_item = this.render_news_item();

    return <div class="news-grid">
      <div class="logo hflow">
        <div class="vflow vcenter">
          <p style="font-size: 7pt;">Welcome</p>
          <h1 class="change" overlay="username">
            {globalThis.username}
          </h1>
        </div>
      </div>
      <div class="left-margin"></div>
      <div class="right-margin"></div>
      <div class="footer-margin"></div>
      <div class="hflow child-margin expand">
        <div class="news_items_container vflow">
          <div class="titlebar">
            <h3 class="title uppercase">News</h3>
          </div>
          <div class="expand">
            { news_items }
          </div>
        </div>
        <div class="news_container vflow">
          <div class="titlebar">
            <h3 class="title">{this.current_news_title}</h3>
          </div>
          <div>
            <div id="news">
              { news_item }
            </div>
          </div>
        </div>
      </div>
    </div>
  }

  render_news_feed() {
    var list = [];
    for (const item of globalThis.news_items) {
      var date = new Date(item.pubDate);
      var type_string = "General";
      if (item.title.match(/\sPATCH\s/i))
        type_string = "Patch";

      list.push(<div class="news_item hflow" id={item.id}>
        <pubDate><day> { (date.getDay() < 10 ? '0' : '') + date.getDay() }</day><month>{ date.toUTCString().split(' ')[2] }</month></pubDate>
        <div class="vflow">
          <p class="news_type">{type_string}</p>
          <p class="news_title">{item.title}</p>
        </div>
      </div>);
    }
    return list;
  }

  render_news_item() {
    return <div></div>
  }
}