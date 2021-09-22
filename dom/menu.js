export class Menu extends Element {
    render(props) {
        return <div class="menuEntries">
        <div page="news.htm">NEWS</div>
        <div class="current" page="game.htm">PLAY GAME</div>
        <div overlay="settings.htm">SETTINGS</div>
      </div>
    }
}