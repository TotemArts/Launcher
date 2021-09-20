class ServerCaption extends Element 
{
  channel = null;

  constructor(props) {
    console.log("ServerCaption for: ");
    console.log(props);
    super();
    this.channel = props.channel;
  }

  render(props) {
    let isCurrent = props.current;
    return <div title={props.key} current={isCurrent}>{props.server.name}</div>;
  }
}

export class ServerList extends Element 
{
  constructor(props) {
    super();
    this.servers = props.servers;
    console.log(this.servers.length);
  }

  render(props) 
  {
    let currentServer = props.current; // ChannelDriver
    var list = Object.values(this.servers).map( (server) => <ServerCaption key={server.data.IP + ":" + server.data.Port} server={server} current={ server === currentServer } /> );
    //       <section id="channel-list" styleset="servers.css#servers" >{ list }</section>

    return <div id="not_chat" class="join_server">
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
      <div class="servers">
        {list}
      </div>
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