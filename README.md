# 🦀 rusty-wit-gui

🔧 A simple WBFS Tool using WIT

⤵️ This program automatically downloads WIT from https://wit.wiimm.de/

❤️ This program works thanks to [Wiimm](https://github.com/Wiimm)'s tools!

## 💾 Setting up the drive

⚠️ Obviously this will delete ALL data on the device

### 🍏 MacOS

- Open the Disk Utility app (Applications -> Utilities)
- Use CMD+2 to make sure physical devices are visible
- Pick the USB drive from the sidebar
- From the toolbar select Erase
- Name the drive a meaningful name (like WII), please make sure the format is "MS-DOS (FAT)" and the scheme is "Master Boot Record"
- Click on the "Erase" button

### 🪟 Windows

- Download Rufus from https://rufus.ie/, choose the portable version
- On "Device" select your drive
- On "Boot selection" pick "Not bootable"
- Choose a meaningful name (like WII) and put it into the "Volume label" box
- On "File System" pick "FAT32"
- Click on the "START" button

### 🐧 Linux (GNOME)

- Open the Disks app
- Click on your drive in the left sidebar
- Click on the menu (three vertical dots in the top-left of the window) and select "Format Disk"
- Make sure Erase is set to Quick and Partitioning is set to MBR/DOS and click "Format"
- Under "Volumes" for your device, click on the "+" button
- Click "Next"
- Choose a meaningful name (like WII) and put it into the "Volume Name" box
- On "Type", choose "For use with all systems and devices (FAT)
- Click "Next" and then "Format"

### 🐧 Linux (KDE)

- Open KDE Partition Manager
- Click on your device in the left sidebar
- Click on "New Partition Table"
- Select "MS-Dos" and click on "Create New Partition Table"
- Click on "unallocated" and then on "New"
- On "File System" select fat32
- Choose a meaningful name (like WII) and put it into the "Label" box
- Click "OK" and then "Apply"
