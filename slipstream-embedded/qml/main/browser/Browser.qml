import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window

import engineering.fizzy.slipstream_embedded

FlexboxLayout {
    id: root
    required property EngineBridge engine

    Layout.fillWidth: true
    Layout.fillHeight: true
    direction: FlexboxLayout.Row
    justifyContent: FlexboxLayout.JustifyStart
    alignItems: FlexboxLayout.AlignCenter
    clip: true

    gap: 8

    FlexboxLayout {
        Layout.fillWidth: false
        Layout.preferredWidth: 48
        Layout.fillHeight: true
        direction: FlexboxLayout.Column
        justifyContent: FlexboxLayout.JustifyStart
        alignItems: FlexboxLayout.AlignCenter
        clip: true

        Item {
            Layout.preferredHeight: 8
        }

        Rectangle {
            Layout.preferredWidth: 48
            Layout.preferredHeight: 48

            color: root.engine.browser_page == BrowserPage.Search ? palette.accent : palette.base

            TapHandler {
                onTapped: root.engine.select_browser_page(BrowserPage.Search)
            }

            Image {
                anchors.centerIn: parent
                width: 24
                height: 24
                sourceSize.width: 24
                sourceSize.height: 24

                source: "qrc:/icons/search.svg"
            }
        }

        Rectangle {
            Layout.preferredWidth: 48
            Layout.preferredHeight: 48

            color: root.engine.browser_page == BrowserPage.Track ? palette.accent : palette.base

            TapHandler {
                onTapped: root.engine.select_browser_page(BrowserPage.Track)
            }

            Image {
                anchors.centerIn: parent
                width: 24
                height: 24
                sourceSize.width: 24
                sourceSize.height: 24

                source: "qrc:/icons/track.svg"
            }
        }

        Rectangle {
            Layout.preferredWidth: 48
            Layout.preferredHeight: 48

            color: root.engine.browser_page == BrowserPage.Artist ? palette.accent : palette.base

            TapHandler {
                onTapped: root.engine.select_browser_page(BrowserPage.Artist)
            }

            Image {
                anchors.centerIn: parent
                width: 24
                height: 24
                sourceSize.width: 24
                sourceSize.height: 24

                source: "qrc:/icons/artist.svg"
            }
        }

        Rectangle {
            Layout.preferredWidth: 48
            Layout.preferredHeight: 48

            color: root.engine.browser_page == BrowserPage.Album ? palette.accent : palette.base

            TapHandler {
                onTapped: root.engine.select_browser_page(BrowserPage.Album)
            }

            Image {
                anchors.centerIn: parent
                width: 24
                height: 24
                sourceSize.width: 24
                sourceSize.height: 24

                source: "qrc:/icons/album.svg"
            }
        }

        Rectangle {
            Layout.preferredWidth: 48
            Layout.preferredHeight: 48

            color: root.engine.browser_page == BrowserPage.Key ? palette.accent : palette.base

            TapHandler {
                onTapped: root.engine.select_browser_page(BrowserPage.Key)
            }

            Image {
                anchors.centerIn: parent
                width: 24
                height: 24
                sourceSize.width: 24
                sourceSize.height: 24

                source: "qrc:/icons/key.svg"
            }
        }

        Rectangle {
            Layout.preferredWidth: 48
            Layout.preferredHeight: 48

            color: root.engine.browser_page == BrowserPage.Playlist ? palette.accent : palette.base

            TapHandler {
                onTapped: root.engine.select_browser_page(BrowserPage.Playlist)
            }

            Image {
                anchors.centerIn: parent
                width: 24
                height: 24
                sourceSize.width: 24
                sourceSize.height: 24

                source: "qrc:/icons/playlist.svg"
            }
        }

        Rectangle {
            Layout.preferredWidth: 48
            Layout.preferredHeight: 48

            color: root.engine.browser_page == BrowserPage.History ? palette.accent : palette.base

            TapHandler {
                onTapped: root.engine.select_browser_page(BrowserPage.History)
            }

            Image {
                anchors.centerIn: parent
                width: 24
                height: 24
                sourceSize.width: 24
                sourceSize.height: 24

                source: "qrc:/icons/history.svg"
            }
        }

        Rectangle {
            Layout.preferredWidth: 48
            Layout.preferredHeight: 48

            color: root.engine.browser_page == BrowserPage.Device ? palette.accent : palette.base

            TapHandler {
                onTapped: root.engine.select_browser_page(BrowserPage.Device)
            }

            Image {
                anchors.centerIn: parent
                width: 24
                height: 24
                sourceSize.width: 24
                sourceSize.height: 24

                source: "qrc:/icons/folder.svg"
            }
        }

        Rectangle {
            Layout.preferredWidth: 48
            Layout.fillHeight: true

            color: palette.base
        }
    }

    FlexboxLayout {
        Layout.fillWidth: true
        Layout.fillHeight: true
        direction: FlexboxLayout.Column
        justifyContent: FlexboxLayout.JustifyStart
        alignItems: FlexboxLayout.AlignCenter
        clip: true

        gap: 8

        Item {
            Layout.preferredHeight: 0
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 48

            color: palette.base

            visible: (root.engine.browser_page == BrowserPage.Search)
            	|| (root.engine.browser_page == BrowserPage.Artist && root.engine.artist_selected)
             	|| (root.engine.browser_page == BrowserPage.Album && root.engine.album_selected)
              	|| (root.engine.browser_page == BrowserPage.Key && root.engine.key_selected)
               	|| (root.engine.browser_page == BrowserPage.Playlist)
                || (root.engine.browser_page == BrowserPage.History && false)
                || (root.engine.browser_page == BrowserPage.Device && false)

            FlexboxLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                direction: FlexboxLayout.Column
                justifyContent: FlexboxLayout.JustifyCenter
                alignContent: FlexboxLayout.AlignStretch

                FlexboxLayout {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 48
                    direction: FlexboxLayout.Row
                    justifyContent: FlexboxLayout.JustifyStart
                    alignContent: FlexboxLayout.AlignStretch

                    Item {
                        Layout.preferredWidth: 48
                        Layout.preferredHeight: 48

                        visible: root.engine.browser_page == BrowserPage.Search

                        Image {
                            anchors.centerIn: parent
                            width: 24
                            height: 24
                            sourceSize.width: 24
                            sourceSize.height: 24

                            source: "qrc:/icons/search.svg"
                        }
                    }

                    Item {
                        Layout.preferredWidth: 48
                        Layout.preferredHeight: 48

                        visible: root.engine.browser_page == BrowserPage.Artist

                        TapHandler {
                            onTapped: root.engine.deselect_artist()
                        }

                        Image {
                            anchors.centerIn: parent
                            width: 24
                            height: 24
                            sourceSize.width: 24
                            sourceSize.height: 24

                            source: "qrc:/icons/chevron-left.svg"
                        }
                    }

                    Item {
                        Layout.preferredWidth: 48
                        Layout.preferredHeight: 48

                        visible: root.engine.browser_page == BrowserPage.Album

                        TapHandler {
                            onTapped: root.engine.deselect_album()
                        }

                        Image {
                            anchors.centerIn: parent
                            width: 24
                            height: 24
                            sourceSize.width: 24
                            sourceSize.height: 24

                            source: "qrc:/icons/chevron-left.svg"
                        }
                    }

                    Item {
                        Layout.preferredWidth: 48
                        Layout.preferredHeight: 48

                        visible: root.engine.browser_page == BrowserPage.Key

                        TapHandler {
                            onTapped: root.engine.deselect_key()
                        }

                        Image {
                            anchors.centerIn: parent
                            width: 24
                            height: 24
                            sourceSize.width: 24
                            sourceSize.height: 24

                            source: "qrc:/icons/chevron-left.svg"
                        }
                    }

                    Item {
                        Layout.preferredWidth: 48
                        Layout.preferredHeight: 48

                        visible: root.engine.browser_page == BrowserPage.Playlist

                        TapHandler {
                            onTapped: {
                            	if (root.engine.playlist_tree_can_pop) {
                             		root.engine.pop_playlist_node();
                             	}
                            }
                        }

                        Image {
                            anchors.centerIn: parent
                            width: 24
                            height: 24
                            sourceSize.width: 24
                            sourceSize.height: 24

                            visible: root.engine.playlist_tree_can_pop

                            source: "qrc:/icons/chevron-left.svg"
                        }
                    }

                    Item {
                        Layout.preferredWidth: 48
                        Layout.preferredHeight: 48

                        visible: root.engine.browser_page == BrowserPage.History

                        TapHandler {
                            onTapped: {}
                        }

                        Image {
                            anchors.centerIn: parent
                            width: 24
                            height: 24
                            sourceSize.width: 24
                            sourceSize.height: 24

                            visible: false

                            source: "qrc:/icons/chevron-left.svg"
                        }
                    }

                    Item {
                        Layout.preferredWidth: 48
                        Layout.preferredHeight: 48

                        visible: root.engine.browser_page == BrowserPage.Device

                        TapHandler {
                            onTapped: {}
                        }

                        Image {
                            anchors.centerIn: parent
                            width: 24
                            height: 24
                            sourceSize.width: 24
                            sourceSize.height: 24

                            visible: false

                            source: "qrc:/icons/chevron-left.svg"
                        }
                    }

                    TextField {
                    	id: search_field
                    	Layout.preferredWidth: 1000
                        Layout.fillHeight: true
                        verticalAlignment: Text.AlignVCenter

                        visible: root.engine.browser_page == BrowserPage.Search

                        onTextEdited: {
                        	root.engine.setSearch(search_field.text);
                         	root.engine.update_browser();
                        }

                    	text: ""
                     	placeholderText: "Search..."

                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        color: palette.text
                        background: Item {}
                    }

                    Text {
                        Layout.fillHeight: true
                        Layout.preferredWidth: 1000
                        verticalAlignment: Text.AlignVCenter

                        visible: root.engine.browser_page == BrowserPage.Artist

                        text: root.engine.active_artist_name

                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        color: palette.text
                    }

                    Text {
                        Layout.fillHeight: true
                        Layout.preferredWidth: 1000
                        verticalAlignment: Text.AlignVCenter

                        visible: root.engine.browser_page == BrowserPage.Album

                        text: root.engine.active_album_name

                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        color: palette.text
                    }

                    Text {
                        Layout.fillHeight: true
                        Layout.preferredWidth: 1000
                        verticalAlignment: Text.AlignVCenter

                        visible: root.engine.browser_page == BrowserPage.Key

                        text: root.engine.active_key_name

                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        color: palette.text
                    }

                    Text {
                        Layout.fillHeight: true
                        Layout.preferredWidth: 1000
                        verticalAlignment: Text.AlignVCenter

                        visible: root.engine.browser_page == BrowserPage.Playlist

                        text: root.engine.playlist_tree_name

                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        color: palette.text
                    }
                }
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 48

            color: palette.base

            FlexboxLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                direction: FlexboxLayout.Column
                justifyContent: FlexboxLayout.JustifyCenter
                alignContent: FlexboxLayout.AlignStretch

                FlexboxLayout {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 48
                    direction: FlexboxLayout.Row
                    justifyContent: FlexboxLayout.JustifySpaceBetween
                    alignContent: FlexboxLayout.AlignStretch

                    gap: 16

                    Item {
                        Layout.preferredWidth: 16
                    }

                    Item {
                        Layout.preferredWidth: 24
                    }

                    Item {
                        Layout.preferredWidth: 16
                    }

                    Text {
                        Layout.fillHeight: true
                        Layout.preferredWidth: 400
                        verticalAlignment: Text.AlignVCenter

                        text: "Title"

                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        color: palette.text
                    }

                    Text {
                        Layout.fillHeight: true
                        Layout.preferredWidth: 400
                        verticalAlignment: Text.AlignVCenter

                        text: "Artist"

                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        color: palette.text
                    }

                    Text {
                        Layout.fillHeight: true
                        Layout.preferredWidth: 400
                        verticalAlignment: Text.AlignVCenter

                        text: "Album"

                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        color: palette.text
                    }

                    Text {
                        Layout.fillHeight: true
                        Layout.preferredWidth: 400
                        verticalAlignment: Text.AlignVCenter

                        text: "Genre"

                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        color: palette.text
                    }

                    Text {
                        Layout.fillHeight: true
                        Layout.preferredWidth: 50
                        verticalAlignment: Text.AlignVCenter

                        text: "Key"

                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        color: palette.text
                    }

                    Text {
                        Layout.fillHeight: true
                        Layout.preferredWidth: 100
                        verticalAlignment: Text.AlignVCenter

                        text: "Length"

                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        color: palette.text
                    }

                    Text {
                        Layout.fillHeight: true
                        Layout.preferredWidth: 100
                        verticalAlignment: Text.AlignVCenter

                        text: "Tempo"

                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        color: palette.text
                    }

                    FlexboxLayout {
                        Layout.fillWidth: false
                        Layout.preferredWidth: 350
                        direction: FlexboxLayout.Row
                        justifyContent: FlexboxLayout.JustifyEnd
                        alignItems: FlexboxLayout.AlignCenter
                    }

                    Item {
                        Layout.preferredWidth: 16
                    }
                }
            }
        }

        ListView {
            id: listView
            required property EngineBridge engine

            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            highlight: Rectangle {
                color: palette.accent
            }
            highlightMoveDuration: 0
            focus: true

            engine: root.engine

            model: root.engine.browser_index
            delegate: Rectangle {
                id: delegate_root
                required property int index
                required property var entry_type
                required property int entry_number
                required property int node_id
                required property string title
                required property string artist
                required property string original_artist
                required property string remixer
                required property string label
                required property string album
                required property string genre
                required property string key
                required property var duration
                required property real bpm

                width: listView.width
                height: 24

                color: (index % 2 == 0) ? palette.base : palette.alternateBase

                TapHandler {
                    onTapped: listView.currentIndex = index
                }

                FlexboxLayout {
                    anchors.fill: parent
                    direction: FlexboxLayout.Row
                    justifyContent: FlexboxLayout.JustifyStart
                    alignContent: FlexboxLayout.AlignStretch

                    gap: 16

                    Item {
                        Layout.preferredWidth: 16
                    }

                    Item {
                        Layout.preferredWidth: 24
                        Layout.preferredHeight: 24

                        Image {
                            anchors.centerIn: parent
                            width: 20
                            height: 20
                            sourceSize.width: 20
                            sourceSize.height: 20

                            visible: delegate_root.entry_type == BrowserEntryType.Track

                            source: "qrc:/icons/track.svg"
                        }

                        Image {
                            anchors.centerIn: parent
                            width: 20
                            height: 20
                            sourceSize.width: 20
                            sourceSize.height: 20

                            visible: delegate_root.entry_type == BrowserEntryType.Playlist

                            source: "qrc:/icons/playlist.svg"
                        }

                        Image {
                            anchors.centerIn: parent
                            width: 20
                            height: 20
                            sourceSize.width: 20
                            sourceSize.height: 20

                            visible: delegate_root.entry_type == BrowserEntryType.Artist

                            source: "qrc:/icons/artist.svg"
                        }

                        Image {
                            anchors.centerIn: parent
                            width: 20
                            height: 20
                            sourceSize.width: 20
                            sourceSize.height: 20

                            visible: delegate_root.entry_type == BrowserEntryType.Album

                            source: "qrc:/icons/album.svg"
                        }

                        Image {
                            anchors.centerIn: parent
                            width: 20
                            height: 20
                            sourceSize.width: 20
                            sourceSize.height: 20

                            visible: delegate_root.entry_type == BrowserEntryType.Key

                            source: "qrc:/icons/key.svg"
                        }
                    }

                    Item {
                        Layout.preferredWidth: 16
                    }

                    Text {
                        Layout.fillHeight: true
                        Layout.preferredWidth: 400
                        verticalAlignment: Text.AlignVCenter
                        clip: true

                        text: delegate_root.title

                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        color: palette.text
                    }

                    Text {
                        Layout.fillHeight: true
                        Layout.preferredWidth: 400
                        verticalAlignment: Text.AlignVCenter
                        clip: true

                        text: delegate_root.artist

                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        color: palette.text
                    }

                    Text {
                        Layout.fillHeight: true
                        Layout.preferredWidth: 400
                        verticalAlignment: Text.AlignVCenter
                        clip: true

                        text: delegate_root.album

                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        color: palette.text
                    }

                    Text {
                        Layout.fillHeight: true
                        Layout.preferredWidth: 400
                        verticalAlignment: Text.AlignVCenter
                        clip: true

                        text: delegate_root.genre

                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        color: palette.text
                    }

                    Text {
                        Layout.fillHeight: true
                        Layout.preferredWidth: 50
                        verticalAlignment: Text.AlignVCenter
                        clip: true

                        text: delegate_root.key

                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        color: palette.text
                    }

                    Text {
                        Layout.fillHeight: true
                        Layout.preferredWidth: 100
                        verticalAlignment: Text.AlignVCenter
                        clip: true

                        visible: delegate_root.entry_type == BrowserEntryType.Track

                        text: {
                            let time_seconds = delegate_root.duration / 1000000000;

                            let duration_minutes = Math.floor((time_seconds / 60) % 100);
                            let duration_seconds = Math.floor(time_seconds % 60);

                            return `${duration_minutes.toString()}:${duration_seconds.toString().padStart(2, "0")}`;
                        }

                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        color: palette.text
                    }

                    Text {
                        Layout.fillHeight: true
                        Layout.preferredWidth: 100
                        verticalAlignment: Text.AlignVCenter
                        clip: true

                        visible: delegate_root.entry_type == BrowserEntryType.Track

                        text: `${delegate_root.bpm.toFixed(1)} bpm`

                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        color: palette.text
                    }

                    Item {
                        Layout.fillWidth: true
                    }

                    FlexboxLayout {
                    	Layout.fillHeight: true
                     	Layout.preferredWidth: 350

                        direction: FlexboxLayout.Row
                        justifyContent: FlexboxLayout.JustifyEnd
                        alignItems: FlexboxLayout.AlignCenter

                        visible: listView.currentIndex == index

                        gap: 8

                        Button {
                            Layout.fillHeight: true

                            visible: delegate_root.entry_type != BrowserEntryType.Track

                            text: " Select "
                            font.pointSize: 12
                            font.variableAxes: {
                                "opsz": 10
                            }
                            font.weight: Font.Medium

                            background: Rectangle {color: palette.light}

                            onClicked: {
                            	if (delegate_root.entry_type == BrowserEntryType.Playlist) {
                             		root.engine.push_playlist_node(delegate_root.node_id);
                             	} else if (delegate_root.entry_type == BrowserEntryType.Artist) {
                              		root.engine.select_artist(delegate_root.node_id);
                              	} else if (delegate_root.entry_type == BrowserEntryType.Album) {
                               		root.engine.select_album(delegate_root.node_id);
                               	} else if (delegate_root.entry_type == BrowserEntryType.Key) {
                                	root.engine.select_key(delegate_root.node_id);
                                }
                            }
                            enabled: listView.engine.device_selected
                        }

                        Button {
                            Layout.fillHeight: true

                            visible: delegate_root.entry_type == BrowserEntryType.Track

                            text: " Load on 1 "
                            font.pointSize: 12
                            font.variableAxes: {
                                "opsz": 10
                            }
                            font.weight: Font.Medium

                            background: Rectangle {color: palette.light}

                            onClicked: () => {
                                listView.engine.load_track(0, listView.engine.active_device, delegate_root.node_id);
                            }
                            enabled: listView.engine.device_selected
                        }
                        Button {
                            Layout.fillHeight: true

                            visible: delegate_root.entry_type == BrowserEntryType.Track

                            text: " Load on 2 "
                            font.pointSize: 12
                            font.variableAxes: {
                                "opsz": 10
                            }
                            font.weight: Font.Medium

                            background: Rectangle {color: palette.light}

                            onClicked: () => {
                                listView.engine.load_track(1, listView.engine.active_device, delegate_root.node_id);
                            }
                            enabled: listView.engine.device_selected
                        }
                        Button {
                            Layout.fillHeight: true

                            visible: delegate_root.entry_type == BrowserEntryType.Track

                            text: " Load on 3 "
                            font.pointSize: 12
                            font.variableAxes: {
                                "opsz": 10
                            }
                            font.weight: Font.Medium

                            background: Rectangle {color: palette.light}

                            onClicked: () => {
                                listView.engine.load_track(2, listView.engine.active_device, delegate_root.node_id);
                            }
                            enabled: listView.engine.device_selected
                        }
                        Button {
                            Layout.fillHeight: true

                            visible: delegate_root.entry_type == BrowserEntryType.Track

                            text: " Load on 4 "
                            font.pointSize: 12
                            font.variableAxes: {
                                "opsz": 10
                            }
                            font.weight: Font.Medium

                            background: Rectangle {color: palette.light}

                            onClicked: () => {
                                listView.engine.load_track(3, listView.engine.active_device, delegate_root.node_id);
                            }
                            enabled: listView.engine.device_selected
                        }
                    }
                }
            }
        }
    }
}
