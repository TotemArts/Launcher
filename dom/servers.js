class VirtualList extends Element {

  currentItem = null; // item, one of items
  selectedItems;// TODO: = new WeakSet();
  styleSet;
  props;

  this(props) {

    let {renderItem,renderList,...rest } = props;
    super.this?.(rest);
    this.props = rest; 
    this.renderItem = renderItem || this.renderItem;
    this.renderList = renderList || this.renderList;
    this.styleset = props.styleset || (__DIR__ + "virtual-select.css#virtual-select");
  }

  itemAt(at) {     // virtual function, must be overriden
    return null;
  }
  totalItems() {   // virtual function, must be overriden
    return 0; 
  }
  indexOf(item) {  // virtual function, must be overriden
    return -1;
  }

  render() {
    let list = [];
    if(!this.vlist) {
      console.log("this isn't a virtual list");
      return this.renderList(list);
    }
    console.log("this is a virtual list");
    let firstIndex = this.vlist.firstBufferIndex;
    let lastIndex = this.vlist.lastBufferIndex;
    let firstVisibleIndex = firstIndex + this.vlist.firstVisibleItem?.elementIndex || 0;
    let lastVisibleIndex = firstIndex + this.vlist.lastVisibleItem?.elementIndex;

    let totalItems = this.totalItems();

    if(this.vlist.itemsTotal != totalItems) { // number of items reduced, update scroll
      if( firstVisibleIndex == 0 ) {  
        this.post(() => {this.vlist.navigate("start")});
        return this.renderList([]); // render empty list and request "from start" navigation
      }
      if( lastVisibleIndex >= totalItems ) {  
        this.post(() => {this.vlist.navigate("end")});
        return this.renderList([]); // render empty list and request "from end" navigation
      }
      lastIndex = Math.min(totalItems, firstIndex + this.vlist.slidingWindowSize) - 1;
      this.post( () => { this.vlist.itemsAfter = totalItems - this.vlist.itemsBefore - this.children.length; });
    }

    let {currentItem, selectedItems } = this;
    for( let index = firstIndex; index <= lastIndex; ++index ) {
      let item = this.itemAt(index);
      if(item) list.push(this.renderItem(item,item === currentItem, selectedItems?.has(item)));
    }
    return this.renderList(list);
  }

  // scroll down
  appendElements(index,n) 
  {
    console.log("appendElements; index: " + index + ", n: " + n);

    let {currentItem, selectedItems } = this;
    if( index === undefined ) index = 0;
    let elements = [];
    for(let i = 0; i < n; ++i, ++index) {
      if(index >= this.totalItems()) break;
      let item = this.itemAt(index);
      elements.push( this.renderItem(item,item === currentItem, selectedItems?.has(item)) );
    }
    this.append(elements);
    return { moreafter: (this.totalItems() - index) }; // return estimated number of items below this chunk
  }

  // scroll up
  prependElements(index,n) 
  {
    console.log("prependElements; index: " + index + ", n: " + n);
    let {currentItem, selectedItems } = this;
    if( index === undefined ) index = this.totalItems() - 1;
    let elements = [];
    for(let i = 0; i < n; ++i, --index) {
      if(index < 0) break;
      let item = this.itemAt(index);
      elements.push( this.renderItem(item,item === currentItem, selectedItems?.has(item)) );
    }
    elements.reverse();
    this.prepend(elements);
    return { morebefore: (index < 0 ? 0 : index + 1) }; // return estimated number of items above this chunk
  }

  // scroll to
  replaceElements(index,n) 
  {
    console.log("replaceElements; index: " + index + ", n: " + n);
    let {currentItem, selectedItems } = this;
    let elements = [];
    let start = index;
    for(let i = 0; i < n; ++i, ++index) {
      if(index >= this.totalItems()) break;
      let item = this.itemAt(index);
      elements.push( this.renderItem(item,item === currentItem, selectedItems?.has(item)) );
    }
    this.patch(elements);
    return { 
      morebefore: start <= 0 ? 0 : start,
      moreafter:  this.totalItems() - index
    }; // return estimated number of items before and above this chunk
  }

  renderList(items) // overridable
  {
    console.log("renderList");
    console.log(items);
    return <table class="servers" {...this.props}>{ items }</table>; 
  }

  renderItem(item,index) // overridable
  {
    return <option key={index}>item { index }</option>;
  }
  
  oncontentrequired(evt)
  {
    let {length, start, where} = evt.data;
    console.log("oncontentrequired; evt.data.length: " + length + ", start: " + start);

    if(where > 0) evt.data = this.appendElements(start,length);  // scrolling down, need to append more elements
    else if(where < 0) evt.data = this.prependElements(start,length); // scrolling up, need to prepend more elements
    else evt.data = this.replaceElements(start,length); // scrolling to index
    return true;
  }

  itemOfElement(element) {
    return this.itemAt(element.elementIndex + this.vlist.firstBufferIndex);
  }

  onkeydown(evt) {
    switch(evt.code) {
      case "KeyDOWN" : 
        if(!this.currentItem) { 
          this.componentUpdate({ currentItem : this.itemOfElement(this.vlist.firstVisibleItem) });
        } else {
          let index = this.indexOf(this.currentItem);
          if( ++index < this.totalItems() ) {
            this.componentUpdate({ currentItem : this.itemAt(index) });
            this.vlist.navigate("advance",index);
          }
        }
        break;
      case "KeyUP" : 
        if(!this.currentItem) { 
          this.componentUpdate({ currentItem : this.itemOfElement(this.vlist.lastVisibleItem) });
        } else {
          let index = this.indexOf(this.currentItem);
          if( --index >= 0 ) {
            this.componentUpdate({ currentItem : this.itemAt(index) });
            this.vlist.navigate("advance",index);
          }
        }
        break;
      case "KeyEND" : 
        this.currentItem = this.itemAt(this.totalItems() - 1);
        this.vlist.navigate("end");
        break;
      case "KeyHOME" : 
        this.currentItem = this.itemAt(0);
        this.vlist.navigate("start");
        break;
      default:
        return false;
    }
    this.post(new Event("input", {bubbles:true}));
    return true;
  }

  setCurrentOption(child) {
    let option;
    for(let node = child; node; node = node.parentElement) {
      if(node.parentElement === this) {
        option = node;
        break; 
      }
    }
    if(option) {
      this.componentUpdate({ currentItem : this.itemOfElement(option) });
      this.post(new Event("input", {bubbles:true}));
      return true;
    }
  }

  ["on mousedown"](evt) { if(evt.button == 1) this.setCurrentOption(evt.target); }
  ["on mousemove"](evt) { if(evt.button == 1) this.setCurrentOption(evt.target); }

  get value() {
    if(!this.currentItem) return undefined;
    return this.currentItem;
  }

}

class List extends VirtualList {
  list; // array of items

  this(props) { 
    let {list, ...rest} = props;
    super.this(rest); 
    this.list = list;
  }

  itemAt(at) {
    console.log("this.list.length: " + this.list.length);

    return this.list[at];
  }
  totalItems() {
    console.log("this.list.length: " + this.list.length);
    return this.list.length;
  }
  indexOf(item) {
    console.log("this.list.length: " + this.list.length);

    return this.list.indexOf(item);
  }

  renderItem(item, isCurrent, isSelected) {
    return <option key={item.key} 
                   state-current={isCurrent} 
                   state-checked={isSelected}>{item.text}</option>;
  }
}

export class Servers extends Element 
{
  list = [
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}];

  render(props) 
  {
    //let currentServer = props.current; // ChannelDriver
    //var list = Object.values(this.servers).map( (server) => <ServerCaption key={server.data.IP + ":" + server.data.Port} server={server} current={ server === currentServer } /> );
    //         {list}

    return <div {...this.props} id="not_chat" class="join_server">
    <div class="titlebar">
      <h3 class="title">Servers</h3>
      <p class="nowrap padding" style="font-size: 7pt;">There are currently <output players_online/> players online</p>
      <div class="spacer"></div>
      filter
      <div class="filter down"></div>
      <div class="refresh" onclick="Window.this.xcall('get_servers', getServersCallback);"></div>
    </div>
    <div class="filterbar">
      <p class="nowrap">Players: <output server_filter_players_min /> - <output server_filter_players_max /></p>
      <div class="slider" minValue="0" maxValue="64" min="0" max="64">
        <div class="range" style="left: 0%; width: *; margin-right: 6dip;"></div>
        <div class="handle start" style="left: -3dip;"></div>
        <div class="handle end" style="left: auto; right: 0dip; margin-right: 0dip;"></div>
      </div>
      <checkmark class="big checked" toggle/><p class="nowrap">Same version</p>
    </div>
    <div class="body mheight">
      <List id="servers" list={this.list} />
    </div>
    <div class="titlebar">
      <h3 class="title"><output title_menu/></h3>
      <div class="spacer"></div>
      <div class="dropdown_menu closed" onclick="this.classList.toggle('open');this.classList.toggle('closed'); document.$('div.body > div.menu').style['visibility'] = this.classList.contains('closed')?'collapse':'visible';">PLAY ONLINE</div>
    </div>
    <div class="body hflow">
      <div class="menu child-padding" style="visibility: hidden;">
        <div class="padding" overlay="ip.htm"><h4>JOIN IP</h4></div>
        <div class="padding"><h4>LAUNCH SERVER</h4></div>
        <div class="padding"><h4>PLAY ONLINE</h4></div>
        <div class="padding"><h4>PLAY SKIRMISH</h4></div>
      </div>
      <div class="expand" style="margin-right: 10px; ">
        <h3>MAP: <span id="map-name" style="color: #CE5135;"></span></h3>
        <div class="hflow" style=" height: *; vertical-align: bottom;">
          <div class="vflow expand child-padding">
            <p>Time Limit: <span id="time-limit"></span></p>
            <p>Vehicle Limit: <span id="vehicle-limit"></span></p>
            <p>Player Limit: <span id="player-limit"></span></p>
            <p>Mine Limit: <span id="mine-limit"></span></p>
            <p>Game Mode: <span id="game-mode"></span></p>
          </div>
          <div class="vflow expand child-padding">
            <p><checkmark class="checked" id="crates"/>Crates</p>
            <p><checkmark class="checked" id="steam"/>Steam Required</p>
            <p><checkmark class="checked" id="ranked"/>Ranked</p>
            <p><checkmark class="checked" id="balance"/>Auto Balance</p>
            <p><checkmark class="checked" id="infantry"/>Infantry Only</p>
          </div>
        </div>
        <button class="green" style="bottom: 0px;" onclick="joinServer();">JOIN SERVER</button>
      </div>
      <video id="map_video" src="../../PreviewVids/Default.avi" loop/>
    </div>
  </div>
  }
}