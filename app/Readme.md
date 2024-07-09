# iOS bindings


## Build

```sh
cargo install tauri-cli --version "^2.0.0-beta.21"
```

See

https://v2.tauri.app/start/prerequisites/

### Android

```sh
export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"
cargo android build
```

```sh
adb devices

```


### iOS

Initialize iOS project

```sh
cargo tauri ios init
```

Open it in XCode and make sure it works (you likely need to fix issues around codesigning).

```sh
cargo tauri ios open
```

Once it works in xcode you can close it and build using cargo from then on:

```sh
cargo tauri ios build
```

Installing on paired development device can be done use CLI:

```sh
xcrun devicectl device install app --device $DEVICE ./gen/apple/build/arm64/delegated-spartan.ipa
```

### Preliminaries

```sh
cargo tauri android init
cargo tauri ios init
```

### Build and run bench bin

```sh
cargo tauri dev
cargo tauri android dev
cargo tauri ios dev
```


```sh
cargo tauri ios dev
```

```sh
xcrun devicectl list devices
xcrun devicectl manage pair --device "Anon’s iPhone"
```

https://stackoverflow.com/questions/5160863/how-to-re-sign-the-ipa-file
```
security find-identity -v -p codesigning
codesign -f -s "ASDASDASD" Payload/*.app
```

```
cargo install --git https://github.com/tauri-apps/cargo-mobile2
```

```
cargo tauri ios build                                                                                              
xcrun devicectl device install app --device $DEVICE ./gen/apple/build/arm64/delegated-spartan.ipa
```


```sh
cargo tauri ios build
xcrun devicectl list devices
xcrun devicectl manage pair --device $DEVICE
xcrun devicectl device install app --device $DEVICE ./gen/apple/build/arm64/delegated-spartan.ipa
```

```sh
cp ./gen/apple/build/arm64/delegated-spartan.ipa ./app.ipa
codesign -vvv ./app.ipa
codesign -s $KEY ./app.ipa
codesign -vvv ./app.ipa
xcrun devicectl device install app --device $DEVICE ./app.ipa
```


This seems to fail on the signature, resigning: <https://stackoverflow.com/questions/5160863/how-to-re-sign-the-ipa-file>

```sh
mkdir tmp; pushd tmp
unzip -q "../gen/apple/build/arm64/delegated-spartan.ipa"
rm -rvf Payload/*.app/_CodeSignature
cp ../*.mobileprovision Payload/*.app/embedded.mobileprovision
/usr/bin/codesign -f -s $KEY Payload/*.app
zip -qr ../app.ipa Payload
popd; rmdir tmp
```

Find keys using

```sh
security find-identity -v -p codesigning
```
