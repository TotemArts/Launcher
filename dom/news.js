export class News extends Element {
    constructor(props, kids) {
        super(props, kids);
        console.log("hi");
    }

    render() {
        return <div class="news-grid">
            <div class="logo hflow">
                <div class="vflow vcenter">
                    <p style="font-size: 7pt;">Welcome</p>
                    <h1 class="change" overlay="username.htm">
                        <output username />
                    </h1>
                </div>
                <div style="width:*;"></div>
                <div class="vflow vcenter" style="text-align: right;">
                    <p style="font-size: 7pt; line-height:100%;">Version</p>
                    <h1><output game_version /></h1>
                </div>
            </div>
            <div class="left-margin"></div>
            <div class="right-margin"></div>
            <div class="footer-margin"></div>
            <div class="hflow child-margin expand">
                <div class="news_items_container vflow">
                    <div class="titlebar">
                        <h3 class="title uppercase">Renegade-X news</h3>
                    </div>
                    <div class="expand">
                    </div>
                </div>
                <div class="news_container vflow">
                    <div class="titlebar">
                        <h3 class="title"><output current_news_title /></h3>
                    </div>
                    <div>
                        <div id="news"></div>
                    </div>
                </div>
            </div>
        </div>
    }
}