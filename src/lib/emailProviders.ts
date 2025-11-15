export interface EmailProvider {
  id: string;
  name: string;
  logo: string; // Base64 encoded image
  smtp_server: string;
  smtp_port: number;
  use_tls: boolean;
  description: string;
  steps: string[];
  helpUrl?: string;
}

export const emailProviders: EmailProvider[] = [
  {
    id: 'gmail',
    name: 'Gmail',
    logo: 'data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCA0OCA0OCI+PHBhdGggZmlsbD0iI2VhNDMzNSIgZD0iTTI0IDE4TDIgMTB2MjZjMCAyLjIgMS44IDQgNCA0aDE2VjI3LjVsLTEuNS0xLjVMMjQgMTh6Ii8+PHBhdGggZmlsbD0iI2ZiYmMwNSIgZD0iTTQyIDEwdjI2YzAgMi4yLTEuOCA0LTQgNEgyMlYyNi41bDEuNS0xLjVMMjQgMTggNDIgMTB6Ii8+PHBhdGggZmlsbD0iIzM0YTg1MyIgZD0iTTQyIDEwdjRMMjQgMjYgNiAxNHYtNGMwLTIuMiAxLjgtNCA0LTRoMjhjMi4yIDAgNCAxLjggNCA0eiIvPjxwYXRoIGZpbGw9IiNjNTIyMWYiIGQ9Ik0yMiAyNi41djE0bDIuNS0yLjUgMS41LTEuNSAxLjUgMS41IDIuNSAyLjV2LTE0TDI0IDMxbC02LTQuNXoiLz48cGF0aCBmaWxsPSIjZmZmIiBkPSJNNCAzNGgxNnY2SDR6TTI4IDM0aDE2djZIMjh6Ii8+PHBhdGggZmlsbD0iIzQyODVmNCIgZD0iTTYgMTRsMTggMTIgMTgtMTJ2LTRINnY0eiIvPjwvc3ZnPg==',
    smtp_server: 'smtp.gmail.com',
    smtp_port: 465,
    use_tls: true,
    description: 'Gmail verwendet Port 465 mit SSL. Für Gmail benötigen Sie ein App-Passwort (nicht Ihr normales Passwort).',
    steps: [
      'Melden Sie sich in Ihrem Google-Konto an',
      'Gehen Sie zu "Sicherheit" → "2-Faktor-Authentifizierung" (muss aktiviert sein)',
      'Scrollen Sie zu "App-Passwörter"',
      'Wählen Sie "Mail" und "Anderes Gerät" aus',
      'Kopieren Sie das generierte 16-stellige Passwort',
      'Verwenden Sie dieses Passwort im Feld "Passwort" (nicht Ihr Gmail-Passwort)',
      'Wichtig: Port 465 und TLS aktiviert lassen',
    ],
    helpUrl: 'https://support.google.com/accounts/answer/185833',
  },
  {
    id: 'outlook',
    name: 'Outlook / Microsoft 365',
    logo: 'data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCI+CjxwYXRoIGZpbGw9IiMwMDc4ZDQiIGQ9Ik0yNCAxMmwtNC41LTQuNXYzSDEydjNoNy41djNMMjQgMTJ6Ii8+CjxwYXRoIGZpbGw9IiMwMDY0YmQiIGQ9Ik0xOS41IDcuNXYzSDE1di0zaDQuNXpNMTUgMTMuNWg0LjV2LTNIMTV2M3oiLz4KPHBhdGggZmlsbD0iIzAwNThiMCIgZD0iTTEwLjUgNkgxNXY2aC00LjVWNnpNMTUgMTJoLTQuNXY2SDE1di02eiIvPgo8cGF0aCBmaWxsPSIjMDA0ZTk1IiBkPSJNNiA2aDQuNXY2SDZWNnpNNiAxMmg0LjV2Nkg2di02eiIvPgo8cGF0aCBmaWxsPSIjMDA0Mjc4IiBkPSJNMS41IDZINnY2SDEuNVY2ek0xLjUgMTJINnY2SDEuNXYtNnoiLz4KPC9zdmc+',
    smtp_server: 'smtp.office365.com',
    smtp_port: 587,
    use_tls: true,
    description: 'Für Microsoft 365 / Outlook.com verwenden Sie Ihre normale Email und Passwort.',
    steps: [
      'Verwenden Sie Ihre vollständige Email-Adresse als Benutzername',
      'Verwenden Sie Ihr normales Outlook/Microsoft-Passwort',
      'Falls 2FA aktiviert ist, erstellen Sie ein App-Passwort unter "Sicherheit" in Ihrem Microsoft-Konto',
      'SMTP muss in Ihren Outlook-Einstellungen aktiviert sein (standardmäßig aktiv)',
    ],
    helpUrl: 'https://support.microsoft.com/de-de/office/pop-imap-und-smtp-einstellungen-8361e398-8af4-4e97-b147-6c6c4ac95353',
  },
  {
    id: 'gmx',
    name: 'GMX',
    logo: 'data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAxMDAgMzAiPgo8cmVjdCB3aWR0aD0iMTAwIiBoZWlnaHQ9IjMwIiBmaWxsPSIjMDA1Njk5Ii8+Cjx0ZXh0IHg9IjUwIiB5PSIyMCIgZm9udC1mYW1pbHk9IkFyaWFsLCBzYW5zLXNlcmlmIiBmb250LXNpemU9IjIwIiBmb250LXdlaWdodD0iYm9sZCIgZmlsbD0iI2ZmZmZmZiIgdGV4dC1hbmNob3I9Im1pZGRsZSI+R01YPC90ZXh0Pgo8L3N2Zz4=',
    smtp_server: 'mail.gmx.net',
    smtp_port: 465,
    use_tls: true,
    description: 'GMX verwendet Port 465 mit SSL. Externe Apps müssen in den Einstellungen aktiviert werden.',
    steps: [
      'Loggen Sie sich bei GMX ein',
      'Gehen Sie zu "Einstellungen" → "Sicherheit"',
      'Aktivieren Sie "Zugriff für weniger sichere Apps" oder erstellen Sie ein App-Passwort',
      'Verwenden Sie Ihre vollständige GMX Email-Adresse als Benutzername',
      'Verwenden Sie Ihr normales Passwort (oder App-Passwort)',
      'Wichtig: Port 465 und TLS aktiviert lassen',
    ],
    helpUrl: 'https://hilfe.gmx.net/pop-imap/einschalten.html',
  },
  {
    id: 'webde',
    name: 'WEB.DE',
    logo: 'data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAxMDAgMzAiPgo8cmVjdCB3aWR0aD0iMTAwIiBoZWlnaHQ9IjMwIiBmaWxsPSIjMDA3YWQwIi8+Cjx0ZXh0IHg9IjUwIiB5PSIyMCIgZm9udC1mYW1pbHk9IkFyaWFsLCBzYW5zLXNlcmlmIiBmb250LXNpemU9IjE4IiBmb250LXdlaWdodD0iYm9sZCIgZmlsbD0iI2ZmZmZmZiIgdGV4dC1hbmNob3I9Im1pZGRsZSI+V0VCLkRFPC90ZXh0Pgo8L3N2Zz4=',
    smtp_server: 'smtp.web.de',
    smtp_port: 465,
    use_tls: true,
    description: 'WEB.DE verwendet Port 465 mit SSL. Externe App-Zugriffe müssen aktiviert werden.',
    steps: [
      'Melden Sie sich bei WEB.DE an',
      'Gehen Sie zu "Einstellungen" → "Sicherheit"',
      'Aktivieren Sie "POP3 & IMAP Abruf" (wenn nicht schon aktiv)',
      'Optional: Erstellen Sie ein App-spezifisches Passwort',
      'Verwenden Sie Ihre vollständige WEB.DE Email-Adresse als Benutzername',
      'Wichtig: Port 465 und TLS aktiviert lassen',
    ],
    helpUrl: 'https://hilfe.web.de/pop-imap/einschalten.html',
  },
  {
    id: 'ionos',
    name: '1&1 IONOS',
    logo: 'data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAxMDAgMzAiPgo8cmVjdCB3aWR0aD0iMTAwIiBoZWlnaHQ9IjMwIiBmaWxsPSIjMDAzMzY2Ii8+Cjx0ZXh0IHg9IjUwIiB5PSIyMCIgZm9udC1mYW1pbHk9IkFyaWFsLCBzYW5zLXNlcmlmIiBmb250LXNpemU9IjE4IiBmb250LXdlaWdodD0iYm9sZCIgZmlsbD0iIzAwOGNkYiIgdGV4dC1hbmNob3I9Im1pZGRsZSI+SU9OT1M8L3RleHQ+Cjwvc3ZnPg==',
    smtp_server: 'smtp.ionos.de',
    smtp_port: 587,
    use_tls: true,
    description: 'Bei IONOS verwenden Sie Ihre Email-Adresse und das zugehörige Passwort.',
    steps: [
      'Verwenden Sie Ihre vollständige IONOS Email-Adresse als Benutzername',
      'Verwenden Sie das Passwort Ihres Email-Postfachs',
      'SMTP ist standardmäßig aktiviert',
      'Bei Problemen prüfen Sie im IONOS Control Center unter "Email" die SMTP-Einstellungen',
    ],
    helpUrl: 'https://www.ionos.de/hilfe/e-mail/e-mail-programme-einrichten/',
  },
  {
    id: 'strato',
    name: 'Strato',
    logo: 'data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAxMDAgMzAiPgo8cmVjdCB3aWR0aD0iMTAwIiBoZWlnaHQ9IjMwIiBmaWxsPSIjZmY2NjAwIi8+Cjx0ZXh0IHg9IjUwIiB5PSIyMCIgZm9udC1mYW1pbHk9IkFyaWFsLCBzYW5zLXNlcmlmIiBmb250LXNpemU9IjE4IiBmb250LXdlaWdodD0iYm9sZCIgZmlsbD0iI2ZmZmZmZiIgdGV4dC1hbmNob3I9Im1pZGRsZSI+U1RSQVRPPC90ZXh0Pgo8L3N2Zz4=',
    smtp_server: 'smtp.strato.de',
    smtp_port: 465,
    use_tls: true,
    description: 'Strato nutzt SSL auf Port 465 (nicht STARTTLS auf 587).',
    steps: [
      'Verwenden Sie Ihre vollständige Strato Email-Adresse als Benutzername',
      'Verwenden Sie das Passwort Ihres Email-Postfachs',
      'Wichtig: Strato nutzt Port 465 mit SSL/TLS',
      'SMTP-Authentifizierung ist erforderlich',
    ],
    helpUrl: 'https://www.strato.de/faq/mail/email-programm-mit-strato-mailserver-verbinden/',
  },
  {
    id: 'telekom',
    name: 'Telekom / T-Online',
    logo: 'data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAxMDAgMzAiPgo8cmVjdCB3aWR0aD0iMTAwIiBoZWlnaHQ9IjMwIiBmaWxsPSIjZTIwMDc0Ii8+Cjx0ZXh0IHg9IjUwIiB5PSIyMCIgZm9udC1mYW1pbHk9IkFyaWFsLCBzYW5zLXNlcmlmIiBmb250LXNpemU9IjE4IiBmb250LXdlaWdodD0iYm9sZCIgZmlsbD0iI2ZmZmZmZiIgdGV4dC1hbmNob3I9Im1pZGRsZSI+VGVsZWtvbTwvdGV4dD4KPC9zdmc+',
    smtp_server: 'securesmtp.t-online.de',
    smtp_port: 587,
    use_tls: true,
    description: 'T-Online Email mit sicherem SMTP-Server.',
    steps: [
      'Verwenden Sie Ihre vollständige T-Online Email-Adresse (mit @t-online.de)',
      'Verwenden Sie Ihr T-Online Email-Passwort',
      'Falls Sie ein Telekom-Login nutzen, verwenden Sie das zugehörige Email-Passwort',
      'SMTP-Authentifizierung ist erforderlich',
    ],
    helpUrl: 'https://www.telekom.de/hilfe/festnetz-internet-tv/e-mail/e-mail-programme-einrichten',
  },
];
