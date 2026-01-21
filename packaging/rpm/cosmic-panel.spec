Name:           cosmic-panel
Epoch:          1
Version:        %{getenv:COSMIC_PANEL_VERSION}
Release:        1%{?dist}
Summary:        COSMIC Panel (Playtron fork)

License:        GPL-3.0-only
URL:            https://github.com/pop-os/cosmic-panel

# No BuildRequires - binary is pre-built

Requires:       dbus
Requires:       (cosmic-notifications >= 1:1.0.0 with cosmic-notifications < 1:1.1.0)

# Override the upstream cosmic-panel
Provides:       cosmic-panel = %{epoch}:%{version}-%{release}
Obsoletes:      cosmic-panel < %{epoch}:%{version}

%description
Panel for the COSMIC desktop environment.
This is the Playtron fork with matching cosmic-notifications-util version
for socket FD communication with cosmic-session.

%prep
%build

%install
# COSMIC_PANEL_SOURCE is set by the build script
install -Dm0755 "%{getenv:COSMIC_PANEL_SOURCE}/target/release/cosmic-panel" "%{buildroot}%{_bindir}/cosmic-panel"

# Install default schema files
find "%{getenv:COSMIC_PANEL_SOURCE}/data/default_schema" -type f | while read -r file; do
    rel_path="${file#%{getenv:COSMIC_PANEL_SOURCE}/data/default_schema/}"
    install -Dm0644 "$file" "%{buildroot}%{_datadir}/cosmic/${rel_path}"
done

install -Dm0644 "%{getenv:COSMIC_PANEL_SOURCE}/LICENSE.md" "%{buildroot}%{_datadir}/licenses/cosmic-panel/LICENSE.md"

%files
%license %{_datadir}/licenses/cosmic-panel/LICENSE.md
%{_bindir}/cosmic-panel
%{_datadir}/cosmic/com.system76.CosmicPanel*

%changelog
* Mon Jan 20 2026 Playtron <dev@playtron.one> - 1.0.2-1
- Initial RPM package for Playtron fork
- Matching cosmic-notifications-util version for socket FD communication
