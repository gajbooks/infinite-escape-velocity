import { Component, Output } from '@angular/core';
import { Subject } from 'rxjs';
import { WebSocketSubject, webSocket } from 'rxjs/webSocket';
import { ClientServerMessage } from 'bindings/ClientServerMessage';
import { ServerClientMessage } from 'bindings/ServerClientMessage';
import { ENVIRONMENT } from 'src/environments/environment';
// @ts-ignore
import * as CBOR from 'cbor-web/dist/cbor';
import { HttpClient, provideHttpClient } from '@angular/common/http';
import { AssetIndexResponse } from 'bindings/AssetIndexResponse';
import { AssetIndexValue } from 'bindings/AssetIndexValue';
import { GameplayCanvasComponent } from './gameplay-canvas/gameplay-canvas.component';
import { CommonModule } from '@angular/common';

function generateWebsocket(url: string): WebSocketSubject<unknown> {
  return webSocket({
    url: url,
    binaryType: 'arraybuffer',
    serializer: (val) => {
      return CBOR.encode(val);
    },
    deserializer: (event) => {
      return CBOR.decode(event.data);
    }
  });
}

function generateWebsocketUrl(): string {
  let prefix = location.protocol == 'https:' ? 'wss://' : 'ws://';
  let host = ENVIRONMENT.PRODUCTION ? location.host : ENVIRONMENT.GAME_SERVER_HOST;
  return `${prefix}${host}/ws`;
}

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [CommonModule, GameplayCanvasComponent],
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.less']
})
export class AppComponent {
  socket = generateWebsocket(generateWebsocketUrl());
  apiBaseUrl = ENVIRONMENT.PRODUCTION ? location.origin : ENVIRONMENT.GAME_SERVER_URL;

  public incomingMessages = new Subject<ServerClientMessage>();
  public outgoingMessages = new Subject<ClientServerMessage>();

  public assetIndex: Promise<AssetIndexValue[]>;

  constructor(private http: HttpClient) {
    let self = this;
    this.assetIndex = new Promise((resolve, reject) => {
      http.get<AssetIndexResponse>(`${this.apiBaseUrl}/assets/index`).subscribe(index => {
        resolve(index.asset_index_list);
        self.subscribeToWebsocket(self);
      });
    })
  }

  disconnectWebsocket() {
    this.outgoingMessages.next({ "type": "Disconnect" });
  }

  subscribeToWebsocket(self: AppComponent) {
    let incomingMessages = self.incomingMessages;
    self.socket.subscribe({
      next(value) {
        if ("type" in <any>value) {
          incomingMessages.next(value as ServerClientMessage);
        } else {
          console.warn("Received garbage type from server: ", value);
        }
      },
      error(err) { console.warn(err) },
      complete() { console.log('disconnected') }
    });

    self.outgoingMessages.subscribe(sent => {
      self.socket.next(sent);
    });
  }
}
