import { Component, inject, Output } from '@angular/core';
import { Subject } from 'rxjs';
import { WebSocketSubject, webSocket } from 'rxjs/webSocket';
import { ClientServerMessage } from 'bindings/ClientServerMessage';
import { ServerClientMessage } from 'bindings/ServerClientMessage';
import { ENVIRONMENT } from 'src/environments/environment';
// @ts-ignore
import * as CBOR from 'cbor-web/dist/cbor';
import { HTTP_INTERCEPTORS, HttpClient, provideHttpClient } from '@angular/common/http';
import { AssetIndexResponse } from 'bindings/AssetIndexResponse';
import { AssetIndexValue } from 'bindings/AssetIndexValue';
import { GameplayCanvasComponent } from './gameplay-canvas/gameplay-canvas.component';
import { CommonModule } from '@angular/common';
import { ChatBoxComponent } from './chat-box/chat-box.component';
import { BaseUrlService } from './services/base-url.service';
import { SessionService } from './services/session.service';

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

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [CommonModule, ChatBoxComponent, GameplayCanvasComponent],
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.less']
})
export class AppComponent {
  private baseUrlService = inject(BaseUrlService);
  socket = generateWebsocket(this.baseUrlService.generateWebsocketUrl());

  private session = inject(SessionService);

  public incomingMessages = new Subject<ServerClientMessage>();
  public outgoingMessages = new Subject<ClientServerMessage>();

  public assetIndex: Promise<AssetIndexValue[]>;

  constructor(private http: HttpClient) {
    let self = this;
    this.assetIndex = new Promise((resolve, reject) => {
      http.get<AssetIndexResponse>("/assets/index").subscribe(index => {
        resolve(index.asset_index_list);
        self.subscribeToWebsocket(self);
      });
    })
  }

  disconnectWebsocket() {
    this.outgoingMessages.next({ "type": "Disconnect" });
  }

  async subscribeToWebsocket(self: AppComponent) {
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

    self.outgoingMessages.next({ "type": "Authorize", "token":  await this.session.getCurrentSessionToken() });
  }
}
