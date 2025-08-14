import { HttpClient, HttpHeaders } from "@angular/common/http";
import { inject, Injectable } from "@angular/core";
import { lastValueFrom, map, Observable } from "rxjs";
import { SessionService } from "./session.service";
import { ChatMessageRequest } from "bindings/players/messaging/ChatMessageRequest";
import { ChatMessageResponse } from "bindings/players/messaging/ChatMessageResponse";
import { BaseUrlService } from "./base-url.service";
import { SseClient } from "ngx-sse-client";

@Injectable({
    providedIn: 'root',
})
export class ChatService {
    private client = inject(HttpClient);
    private session = inject(SessionService);
    private sseClient = inject(SseClient);

    async sendMessage(message: string): Promise<null> {
        let data: ChatMessageRequest = { message: message };
        const request = this.client.post<null>('/players/messaging/send-message', data, {
            headers: {
                'Authorization': await this.session.getCurrentSessionToken()
            }
        });
        return await lastValueFrom(request);
    }

    async subscribeToChat(): Promise<Observable<ChatMessageResponse>> {
        const auth = await this.session.getCurrentSessionToken();
        const headers = new HttpHeaders().set('Authorization', auth);
        return this.sseClient.stream('/players/messaging/subscribe-message', {}, { headers }).pipe(map(event => {
            if (event.type === 'error') {
                const errorEvent = event as ErrorEvent;
                console.error(errorEvent.error, errorEvent.message);
            } else {
                const messageEvent = event as MessageEvent;
                return JSON.parse(messageEvent.data);
            }
        }));
    }
}