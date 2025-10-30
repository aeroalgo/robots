#!/bin/bash

# Скрипт для настройки Wine prefix MT5

export WINEPREFIX="/media/aero/Z/Games/mt5"
export WINEARCH=win64

echo "🍷 Настройка Wine prefix: $WINEPREFIX"
echo ""
echo "Выберите действие:"
echo "1) Открыть winecfg (настройка разрешения и графики)"
echo "2) Установить разрешение 1920x1080"
echo "3) Установить разрешение 1366x768"
echo "4) Установить виртуальный рабочий стол"
echo "5) Отключить виртуальный рабочий стол"
echo ""
read -p "Введите номер: " choice

case $choice in
    1)
        echo "🎨 Открываю winecfg..."
        echo "Перейдите на вкладку Graphics и настройте разрешение"
        WINEPREFIX="$WINEPREFIX" winecfg
        ;;
    2)
        echo "📐 Устанавливаю разрешение 1920x1080..."
        WINEPREFIX="$WINEPREFIX" winetricks vd=1920x1080
        ;;
    3)
        echo "📐 Устанавливаю разрешение 1366x768..."
        WINEPREFIX="$WINEPREFIX" winetricks vd=1366x768
        ;;
    4)
        echo "🖥️ Включаю виртуальный рабочий стол..."
        read -p "Введите разрешение (например, 1920x1080): " resolution
        WINEPREFIX="$WINEPREFIX" winetricks vd=$resolution
        ;;
    5)
        echo "🖥️ Отключаю виртуальный рабочий стол..."
        WINEPREFIX="$WINEPREFIX" winetricks vd=off
        ;;
    *)
        echo "❌ Неверный выбор"
        exit 1
        ;;
esac

echo "✅ Готово!"

