# A service that reads performative data from network routers

import subprocess
import sqlite3
import platform
import statistics
import datetime
import re
import csv
import os
import shutil
import time 
import socket
import psutil
import speedtest
import requests

DB_FILE = "data.db"
BACKUP_FILE = "data_backup.db"

def get_current_timestamp():
    return datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")

def get_active_interface():
    system = platform.system()
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.connect(("8.8.8.8", 80))
        local_ip_address = s.getsockname()[0]
        s.close()

        interfaces = psutil.net_if_addrs()
        for iface_name, iface_addresses in interfaces.items():
            for addr in iface_addresses:
                if addr.address == local_ip_address:
                    return iface_name
    except Exception as e:
        print(f"Error determining active interface: {e}")
    return "Unknown"

def get_default_gateway_ip():
    try:
        system = platform.system()
        if system == "Windows":
            output = subprocess.check_output("ipconfig", shell=True).decode()
            match = re.search(r"Default Gateway[ .:]*([\d.]+)", output)
        else:
            output = subprocess.check_output("ip route", shell=True).decode()
            match = re.search(r"default via ([\d.]+)", output)
        return match.group(1) if match else "unknown"
    except:
        return "unknown"

def get_router_ip():
    try:
        if platform.system() == "Windows":
            output = subprocess.check_output("ipconfig", shell=True).decode()
            match = re.search(r"Default Gateway[ .:]*([\d.]+)", output)
        else:
            output = subprocess.check_output("ip route", shell=True).decode()
            match = re.search(r"default via ([\d.]+)", output)
        if match:
            return match.group(1)
    except Exception as e:
        print(f"[!] Error getting router IP: {e}")
    return "unknown"

def get_router_mac(ip):
    try:
        ping_cmd = f"ping -c 1 {ip}" if platform.system() != "Windows" else f"ping -n 1 {ip}"
        subprocess.run(ping_cmd, shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        arp_output = subprocess.check_output("arp -a", shell=True).decode()
        for line in arp_output.splitlines():
            if ip in line:
                match = re.search(r'(([0-9a-f]{2}[:-]){5}([0-9a-f]{2}))', line.lower())
                if match:
                    return match.group(0)
    except Exception as e:
        print(f"[!] Error getting MAC address: {e}")
    return "unknown"

def ping_router(ip, count=10):
    system = platform.system()
    if system == "Windows":
        command = ["ping", "-n", str(count), ip]
    else:
        command = ["ping", "-c", str(count), ip]

    try:
        output = subprocess.check_output(command, stderr=subprocess.DEVNULL).decode()
        times = [float(t) for t in re.findall(r"time[=<](\d+(?:\.\d+)?)", output)]
        latency = round(statistics.mean(times), 2) if times else None
        jitter = round(statistics.stdev(times), 2) if len(times) > 1 else 0.0
        packet_loss = 100 * (count - len(times)) / count
        return latency, packet_loss, jitter
    except Exception as e:
        print(f"[!] Ping error: {e}")
        return None, None, None

def get_signal_strength():
    try:
        output = subprocess.check_output("netsh wlan show interfaces", shell=True, text=True)
        match = re.search(r"^\s*Signal\s*:\s*(\d+)%", output, re.MULTILINE)
        if match:
            return f"{int(match.group(1))}%"
        else:
            print("[!] Signal strength not found.")
            return "unknown"
    except subprocess.CalledProcessError as e:
        print(f"[!] Error retrieving signal strength: {e}")
        return "unknown"

def get_speedtest():
    try:
        st = speedtest.Speedtest()
        st.get_best_server()
        download_speed = round(st.download() / 1_000_000, 2)  # Mbps
        upload_speed = round(st.upload() / 1_000_000, 2)      # Mbps
        return download_speed, upload_speed
    except Exception as e:
        print(f"[!] Speedtest error: {e}")
        return None, None
    
def get_isp_name():
    try:
        response = requests.get("https://ipinfo.io/json", timeout=10)
        if response.status_code == 200:
            data = response.json()
            isp = data.get("org", "unknown")  # 'org' usually contains ISP info
            return isp
        else:
            print(f"[!] Failed to get ISP info: status code {response.status_code}")
    except Exception as e:
        print(f"[!] Exception while fetching ISP: {e}")
    return "unknown"

def get_default_gateway_ip(): #for checking gateway reachability
    
    try:
        gws = psutil.net_if_stats()
        gateways = psutil.net_if_addrs()
        default_gws = psutil.net_if_stats()
        default_gateways = psutil.net_if_stats()

        gws = psutil.net_if_stats()
        default_gateways = psutil.net_if_stats()
    except:
        pass  # Fall through to system-based fallback

    system = platform.system()

    if system == "Windows":
        # Parse `ipconfig` for default gateway
        try:
            output = subprocess.check_output("ipconfig", stderr=subprocess.DEVNULL).decode()
            match = re.search(r"Default Gateway[ .:]*([\d.]+)", output)
            if match:
                return match.group(1)
        except Exception:
            pass

    elif system in ("Linux", "Darwin"):
        try:
            output = subprocess.check_output(["ip", "route"], stderr=subprocess.DEVNULL).decode()
            match = re.search(r"default via ([\d.]+)", output)
            if match:
                return match.group(1)
        except Exception:
            try:
                output = subprocess.check_output(["route", "-n"], stderr=subprocess.DEVNULL).decode()
                match = re.search(r"^0.0.0.0\s+([\d.]+)", output, re.MULTILINE)
                if match:
                    return match.group(1)
            except Exception:
                pass

    return None  

def check_gateway_reachability(ip, count=10):
    system = platform.system()
    if system == "Windows":
        command = ["ping", "-n", str(count), ip]
    else:
        command = ["ping", "-c", str(count), ip]

    try:
        output = subprocess.check_output(command, stderr=subprocess.DEVNULL).decode()
        received = len(re.findall(r"time[=<](\d+(?:\.\d+)?)", output))
        reachability = round((received / count) * 100)
        return f"{reachability}%"
    except Exception as e:
        print(f"[!] Error checking gateway reachability: {e}")
        return "unknown"

def get_interface_ip():
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.connect(("8.8.8.8", 80))
        ip_address = s.getsockname()[0]
        s.close()
        return ip_address
    except Exception as e:
        print(f"[!] Error getting interface IP: {e}")
        return "unknown"

def log_to_db(timestamp, router_ip, router_mac, latency, packet_loss, jitter, signal_strength, download_speed, upload_speed, isp_name, gw_reach, interface_ip):
    interface = get_active_interface()
    now = datetime.datetime.now()
    date = now.strftime("%Y-%m-%d")
    time_str = now.strftime("%H:%M:%S")

    conn = sqlite3.connect(DB_FILE)
    cursor = conn.cursor()
    cursor.execute("""
        INSERT INTO new_probe_logs (date, time, router_ip, router_mac, interface, latency, jitter, packet_loss, signal_strength, download_speed, upload_speed, isp_name, gw_reach, interface_ip)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    """, (date, time_str, router_ip, router_mac, interface, latency, jitter, packet_loss, signal_strength, download_speed, upload_speed, isp_name, gw_reach, interface_ip))

    conn.commit()
    conn.close()

def initialize_database():
    if os.path.exists(DB_FILE):
        shutil.copy(DB_FILE, BACKUP_FILE)
        print(f"Backed up old database to '{BACKUP_FILE}'.")

    conn = sqlite3.connect(DB_FILE)
    cursor = conn.cursor()
    cursor.execute('''
         CREATE TABLE IF NOT EXISTS new_probe_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT,
            time TEXT,
            router_ip TEXT,
            router_mac TEXT,
            interface TEXT,
            latency REAL,
            jitter REAL,
            packet_loss REAL,
            signal_strength TEXT,
            download_speed REAL,
            upload_speed REAL,
            isp_name TEXT,
            gw_reach REAL,
            interface_ip TEXT
        )
    ''')
    conn.commit()
    conn.close()
    print("Database and table created successfully.")

def export_to_csv(db_path="data.db", csv_path="new_probe_logs.csv"):
    if not os.path.exists(db_path):
        print("[!] Database not found.")
        return

    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    cursor.execute("SELECT * FROM new_probe_logs ORDER BY id DESC LIMIT 1")
    row = cursor.fetchone()

    file_exists = os.path.exists(csv_path)
    with open(csv_path, mode='a', newline='') as csvfile:
        writer = csv.writer(csvfile)
        if not file_exists:
            writer.writerow([
                "ID", "Date", "Time", "Router IP", "Router MAC",
                "Interface", "Latency", "Jitter", "Packet Loss", 
                "Signal Strength", "Download Speed", "Upload Speed",
                "ISP Name", "Gateway Reachability", "Interface IP"

            ])
        if row:
            writer.writerow(row)

    conn.close()
    print(f"[✓] Appended latest record to {csv_path}")

def main():
    initialize_database()
    
    try:
        while True:
            print("[*] Starting network probe...")

            gateway_ip = get_default_gateway_ip()

            timestamp = get_current_timestamp()
            router_ip = get_router_ip()
            router_id = get_router_mac(router_ip)
            latency, packet_loss, jitter = ping_router(router_ip)        
            signal_strength = get_signal_strength()
            download_speed, upload_speed = get_speedtest()
            isp_name = get_isp_name()
            gw_reach = check_gateway_reachability(gateway_ip)
            interface_ip = get_interface_ip()
            if latency is None:
                print("[!] Ping failed, skipping log.")
                time.sleep(50)
                continue

            print(f"[+] Timestamp: {timestamp}")
            print(f"[+] Router IP: {router_ip}")
            print(f"[+] Router ID (MAC): {router_id}")
            print(f"[+] Latency: {latency} ms")
            print(f"[+] Packet Loss: {packet_loss:.2f}%")
            print(f"[+] Jitter: {jitter} ms")
            print(f"[+] Signal Strength: {signal_strength}")
            print(f"[+] Download: {download_speed} Mbps")
            print(f"[+] Upload: {upload_speed} Mbps")
            print(f"[+] ISP: {isp_name}")
            print(f"[+] GW Reachability: {gw_reach}")
            print(f"[+] Interface IP: {interface_ip}")

            log_to_db(timestamp, router_ip, router_id, latency, packet_loss, jitter, signal_strength, download_speed, upload_speed, isp_name, gw_reach, interface_ip)
            print("[✓] Logged successfully.")

            export_to_csv()
            print("[⏳] \n")
            time.sleep(300)  # 5 minutes

    except KeyboardInterrupt:
        print("\n[!] Stopped by user. Exiting...")

if __name__ == "__main__":
    main()
