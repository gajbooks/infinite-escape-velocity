import { HttpClient, HttpHeaders } from "@angular/common/http";
import { inject, Injectable } from "@angular/core";
import { lastValueFrom, map, Observable } from "rxjs";
import { SessionService } from "./session.service";
import { ChatMessageRequest } from "bindings/players/messaging/ChatMessageRequest";
import { ChatMessageResponse } from "bindings/players/messaging/ChatMessageResponse";
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

    async subscribeToChat(): Promise<Observable<ChatMessageResponse | null>> {
        const auth = await this.session.getCurrentSessionToken();
        const headers = new HttpHeaders().set('Authorization', auth);

        // This delay is a really stupid hack because the current working theory is that Firefox gets upset
        // and lags if a SSE request happens before a websocket one (but not after?) TODO: Report to Firefox
        // Only happens on refreshes due to random script timings, but it delays the entire page loading
        await new Promise(r => setTimeout(r, 1000));
        return this.sseClient.stream('/players/messaging/subscribe-message', {}, { headers }).pipe(map(event => {
            if (event.type === 'error') {
                console.error(event);
                return null;
            } else if (event.type === 'message') {
                const messageEvent = event as MessageEvent;
                return JSON.parse(messageEvent.data);
            } else {
                return null;
            }
        }));
    }
}
