import QtQuick
import QtQuick.VirtualKeyboard
import QtQuick.VirtualKeyboard.Styles

KeyboardStyle {
    id: currentStyle

    readonly property string fontFamily: "Helvetica"
    readonly property real keyBackgroundMargin: Math.round(8 * scaleHint)
    readonly property real keyContentMargin: Math.round(40 * scaleHint)

    keyboardDesignWidth: 2560
    keyboardDesignHeight: 800
    keyboardRelativeLeftMargin: 16 / keyboardDesignWidth
    keyboardRelativeRightMargin: 16 / keyboardDesignWidth
    keyboardRelativeTopMargin: 16 / keyboardDesignHeight
    keyboardRelativeBottomMargin: 16 / keyboardDesignHeight

    keyboardBackground: Rectangle {
        color: "#1f1f1f"
    }

    keyPanel: KeyPanel {
        Rectangle {
            anchors.fill: parent
            anchors.margins: keyBackgroundMargin
            radius: 0
            color: control.pressed ? palette.accent : palette.base

            Text {
                anchors.centerIn: parent
                text: (InputContext.shiftActive || InputContext.capsLockActive) ? control.displayText.toUpperCase() : control.displayText
                color: palette.text
                font.family: fontFamily
                font.pixelSize: 64 * scaleHint
            }
        }
    }

    backspaceKeyPanel: KeyPanel {
        Rectangle {
            anchors.fill: parent
            anchors.margins: keyBackgroundMargin
            radius: 0
            color: control.pressed ? palette.accent : palette.base

            Image {
                id: backspaceKeyIcon
                anchors.centerIn: parent
                sourceSize.height: 72 * scaleHint
                smooth: false
                source: "qrc:/icons/delete.svg"
            }
        }
    }

    languageKeyPanel: KeyPanel {
        Rectangle {
            anchors.fill: parent
            anchors.margins: keyBackgroundMargin
            radius: 0
            color: control.pressed ? palette.accent : palette.base

            Image {
                id: languageKeyIcon
                anchors.centerIn: parent
                sourceSize.height: 72 * scaleHint
                smooth: false
                source: "qrc:/icons/globe.svg"
            }
        }
    }

    enterKeyPanel: KeyPanel {
        Rectangle {
            anchors.fill: parent
            anchors.margins: keyBackgroundMargin
            radius: 0
            color: control.pressed ? palette.accent : palette.base

            Text {
                anchors.centerIn: parent
                text: "return"
                color: palette.text
                font.family: fontFamily
                font.pixelSize: 56 * scaleHint
            }
        }
    }

    hideKeyPanel: KeyPanel {
        Rectangle {
            anchors.fill: parent
            anchors.margins: keyBackgroundMargin
            radius: 0
            color: control.pressed ? palette.accent : palette.base

            Image {
                id: hideKeyIcon
                anchors.centerIn: parent
                sourceSize.height: 72 * scaleHint
                smooth: false
                source: "qrc:/icons/keyboard-chevron-down.svg"
            }
        }
    }

    shiftKeyPanel: KeyPanel {
        Rectangle {
            anchors.fill: parent
            anchors.margins: keyBackgroundMargin
            radius: 0
            color: control.pressed ? palette.accent : ((InputContext.shiftActive || InputContext.capsLockActive) ? palette.light : palette.base)

            Image {
                id: shiftKeyIcon
                anchors.centerIn: parent
                sourceSize.height: 72 * scaleHint
                smooth: false
                source: "qrc:/icons/shift.svg"
                visible: !InputContext.shiftActive && !InputContext.capsLockActive
            }

            Image {
                id: shiftActiveKeyIcon
                anchors.centerIn: parent
                sourceSize.height: 72 * scaleHint
                smooth: false
                source: "qrc:/icons/shift-filled-dark.svg"
                visible: InputContext.shiftActive && !InputContext.capsLockActive && !control.pressed
            }

            Image {
                id: shiftActivePressKeyIcon
                anchors.centerIn: parent
                sourceSize.height: 72 * scaleHint
                smooth: false
                source: "qrc:/icons/shift-filled.svg"
                visible: InputContext.shiftActive && !InputContext.capsLockActive && control.pressed
            }

            Image {
                id: shiftCapsKeyIcon
                anchors.centerIn: parent
                sourceSize.height: 72 * scaleHint
                smooth: false
                source: "qrc:/icons/caps-filled-dark.svg"
                visible: InputContext.capsLockActive && !control.pressed
            }

            Image {
                id: shiftCapsPressKeyIcon
                anchors.centerIn: parent
                sourceSize.height: 72 * scaleHint
                smooth: false
                source: "qrc:/icons/caps-filled.svg"
                visible: InputContext.capsLockActive && control.pressed
            }
        }
    }

    spaceKeyPanel: KeyPanel {
        Rectangle {
            anchors.fill: parent
            anchors.margins: keyBackgroundMargin
            radius: 0
            color: control.pressed ? palette.accent : palette.base

            Text {
                anchors.centerIn: parent
                text: "space"
                color: palette.text
                font.family: fontFamily
                font.pixelSize: 56 * scaleHint
            }
        }
    }

    symbolKeyPanel: KeyPanel {
        Rectangle {
            anchors.fill: parent
            anchors.margins: keyBackgroundMargin
            radius: 0
            color: control.pressed ? palette.accent : palette.base

            Text {
                anchors.centerIn: parent
                text: control.displayText
                color: palette.text
                font.family: fontFamily
                font.pixelSize: 56 * scaleHint
            }
        }
    }

    selectionHandle: Item {
        implicitWidth: 44 * scaleHint
        implicitHeight: 56 * scaleHint

        Rectangle {
            id: stem
            width: 3
            height: 0 * scaleHint
            anchors.horizontalCenter: parent.horizontalCenter
            color: palette.light
        }
        Rectangle {
            width: 56 * scaleHint
            height: width
            radius: width / 2
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.top: stem.bottom
            color: palette.light
        }
    }
}
