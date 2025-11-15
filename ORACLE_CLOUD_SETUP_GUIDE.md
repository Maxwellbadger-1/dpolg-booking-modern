# 🚀 ORACLE CLOUD VM SETUP - SCHRITT FÜR SCHRITT
## Gemeinsame Einrichtung für DPolG Booking System

---

## 📋 VORBEREITUNG

### Was wir brauchen:
- [ ] Oracle Cloud Account (Free Tier)
- [ ] SSH Key Pair (erstellen wir gleich)
- [ ] 30 Minuten Zeit

---

## SCHRITT 1: SSH KEYS ERSTELLEN (Lokal auf Ihrem Mac)

Öffnen Sie das Terminal und führen Sie diese Befehle aus:

```bash
# 1. SSH Ordner erstellen (falls nicht vorhanden)
mkdir -p ~/.ssh

# 2. SSH Key Pair generieren (für Oracle Cloud)
ssh-keygen -t rsa -b 4096 -f ~/.ssh/oracle_cloud_dpolg -C "dpolg-booking@oracle"

# WICHTIG: Bei Passwort-Abfrage einfach ENTER drücken (kein Passwort)
```

### Nach Ausführung haben Sie:
- **Private Key:** ~/.ssh/oracle_cloud_dpolg (NIEMALS teilen!)
- **Public Key:** ~/.ssh/oracle_cloud_dpolg.pub (für Oracle Cloud)

```bash
# 3. Public Key anzeigen (brauchen wir gleich für Oracle)
cat ~/.ssh/oracle_cloud_dpolg.pub
```

**KOPIEREN Sie diesen Public Key! Wir brauchen ihn in Schritt 3.**

---

## SCHRITT 2: ORACLE CLOUD LOGIN

1. Gehen Sie zu: https://cloud.oracle.com
2. Loggen Sie sich ein
3. Wählen Sie Ihre Region (am besten Frankfurt - eu-frankfurt-1)

---

## SCHRITT 3: VM INSTANZ ERSTELLEN

### 3.1 Navigation
- Klicken Sie auf **"☰"** (Hamburger Menu)
- **Compute** → **Instances**
- Klicken Sie auf **"Create Instance"**

### 3.2 Basic Information
- **Name:** `dpolg-booking-vm`
- **Create in compartment:** (default compartment ist OK)

### 3.3 Placement
- **Availability domain:** AD-1 (oder was verfügbar ist)
- **Fault domain:** Let Oracle choose (recommended)

### 3.4 Security
- ✅ **Secure Shell (SSH) key**
- ✅ **Generate a key pair for me** ODER
- ✅ **Paste public keys** → Fügen Sie Ihren Public Key ein (von Schritt 1)

### 3.5 Image and Shape

#### Image:
- **Image:** Ubuntu
- **Image version:** 22.04 LTS (Minimal)
- Klicken Sie auf **"Change Image"**
  - Wählen Sie **"Canonical Ubuntu"**
  - Version: **22.04 Minimal**
  - Klicken Sie **"Select Image"**

#### Shape:
- Klicken Sie auf **"Change Shape"**
- **Shape series:** Ampere (ARM-based processor)
- **Shape name:** VM.Standard.A1.Flex
- **Number of OCPUs:** 2
- **Amount of memory (GB):** 12
- Klicken Sie **"Select Shape"**

**WICHTIG:** Falls Ampere nicht verfügbar:
- Alternative: **VM.Standard.E2.1.Micro** (AMD x86)
- 1 OCPU, 1 GB RAM (weniger, aber funktioniert)

### 3.6 Networking
- **Primary network:** Create new virtual cloud network
- **New virtual cloud network name:** `dpolg-vcn`
- **New subnet name:** `dpolg-subnet`
- ✅ **Assign a public IPv4 address**

### 3.7 Add SSH Keys
- Falls noch nicht gemacht: Fügen Sie Ihren Public Key hier ein

### 3.8 Boot Volume
- **Boot volume size (GB):** 50 (default)
- **Use in-transit encryption:** Optional

### 3.9 Create
- Klicken Sie **"Create"**
- WARTEN Sie 2-3 Minuten bis Status = **RUNNING** (grün)

---

## SCHRITT 4: NETZWERK-KONFIGURATION (FIREWALL)

### 4.1 Security List Update
1. Wenn VM läuft, notieren Sie die **Public IP Address**
2. Gehen Sie zu **Networking** → **Virtual Cloud Networks**
3. Klicken Sie auf **dpolg-vcn**
4. Klicken Sie auf **Security Lists**
5. Klicken Sie auf **Default Security List**

### 4.2 Ingress Rules hinzufügen
Klicken Sie auf **"Add Ingress Rules"** und fügen Sie diese Regeln hinzu:

#### Regel 1: PostgreSQL
- **Source Type:** CIDR
- **Source CIDR:** 0.0.0.0/0
- **IP Protocol:** TCP
- **Source Port Range:** (leer lassen)
- **Destination Port Range:** 5432
- **Description:** PostgreSQL

#### Regel 2: pgBouncer
- **Source Type:** CIDR
- **Source CIDR:** 0.0.0.0/0
- **IP Protocol:** TCP
- **Source Port Range:** (leer lassen)
- **Destination Port Range:** 6432
- **Description:** pgBouncer

#### Regel 3: HTTP (optional, für Monitoring)
- **Source Type:** CIDR
- **Source CIDR:** 0.0.0.0/0
- **IP Protocol:** TCP
- **Destination Port Range:** 80
- **Description:** HTTP

---

## SCHRITT 5: SSH VERBINDUNG TESTEN

```bash
# Ersetzen Sie <PUBLIC_IP> mit Ihrer VM IP
ssh -i ~/.ssh/oracle_cloud_dpolg ubuntu@<PUBLIC_IP>

# Beispiel:
ssh -i ~/.ssh/oracle_cloud_dpolg ubuntu@132.145.123.456

# Beim ersten Mal: "yes" eingeben für Fingerprint
```

### Wenn verbunden:
```bash
# Test ob alles funktioniert
ubuntu@dpolg-booking-vm:~$ whoami
# Output: ubuntu

ubuntu@dpolg-booking-vm:~$ df -h
# Zeigt Festplattenspeicher

ubuntu@dpolg-booking-vm:~$ free -m
# Zeigt RAM

# Vorerst: NICHT disconnecten!
```

---

## SCHRITT 6: ZUGRIFF FÜR AUTOMATION EINRICHTEN

### 6.1 Auf der VM (noch in SSH Session):

```bash
# 1. Automation User erstellen
sudo useradd -m -s /bin/bash automation
sudo usermod -aG sudo automation

# 2. SSH Ordner für automation user
sudo mkdir -p /home/automation/.ssh
sudo chown -R automation:automation /home/automation/.ssh
sudo chmod 700 /home/automation/.ssh

# 3. Authorized keys vorbereiten
sudo touch /home/automation/.ssh/authorized_keys
sudo chmod 600 /home/automation/.ssh/authorized_keys
sudo chown automation:automation /home/automation/.ssh/authorized_keys

# 4. Sudo ohne Passwort für automation
echo "automation ALL=(ALL) NOPASSWD:ALL" | sudo tee /etc/sudoers.d/automation
```

### 6.2 Neuen SSH Key für Automation erstellen (auf Ihrem Mac):

**NEUES Terminal-Fenster öffnen!**

```bash
# Automation SSH Key erstellen
ssh-keygen -t ed25519 -f ~/.ssh/oracle_automation_key -C "automation@dpolg" -N ""

# Public Key anzeigen
cat ~/.ssh/oracle_automation_key.pub
```

**KOPIEREN Sie diesen Key!**

### 6.3 Key auf VM hinzufügen (zurück zur SSH Session):

```bash
# Fügen Sie den kopierten Public Key ein:
echo "HIER_DEN_PUBLIC_KEY_EINFÜGEN" | sudo tee -a /home/automation/.ssh/authorized_keys
```

### 6.4 Test der Automation-Verbindung (neues Terminal):

```bash
# Test
ssh -i ~/.ssh/oracle_automation_key automation@<PUBLIC_IP>

# Sollte funktionieren ohne Passwort
```

---

## SCHRITT 7: PRIVATE KEY FÜR MICH VORBEREITEN

### WICHTIG: Private Key sicher teilen

Der Private Key (~/.ssh/oracle_automation_key) muss ich haben, um automatisch auf die VM zuzugreifen.

**So machen wir es sicher:**

```bash
# 1. Private Key anzeigen
cat ~/.ssh/oracle_automation_key

# 2. Kopieren Sie den KOMPLETTEN Output (inkl. BEGIN und END Zeilen)
```

**Der Key sieht so aus:**
```
-----BEGIN OPENSSH PRIVATE KEY-----
[viele Zeichen]
-----END OPENSSH PRIVATE KEY-----
```

---

## 📝 CHECKLISTE - Was ich von Ihnen brauche:

Nach Abschluss dieser Schritte brauche ich:

1. **Public IP der VM:** _______________
2. **Private Key (oracle_automation_key):** (kompletter Inhalt)
3. **Bestätigung dass SSH funktioniert:** JA / NEIN

---

## 🔒 SICHERHEITSHINWEISE

- Der Private Key ist wie ein Passwort - sicher aufbewahren!
- Nach Setup: Keys können rotiert werden
- Die VM ist nur über SSH erreichbar (kein Passwort-Login)

---

## NÄCHSTE SCHRITTE

Sobald ich Zugriff habe, werde ich:
1. PostgreSQL installieren
2. pgBouncer konfigurieren
3. Datenbank-Migration durchführen
4. Backend anpassen
5. Alles testen

---

**Status:** WARTEN AUF IHRE AKTIONEN

Führen Sie die Schritte 1-7 aus und teilen Sie mir dann:
- Die Public IP
- Den Private Key
- Bestätigung dass alles funktioniert

Dann kann ich vollautomatisch weitermachen!