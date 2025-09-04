import {inject, Injectable} from '@angular/core';
import {HttpEvent, HttpInterceptor, HttpHandler, HttpRequest} from '@angular/common/http';
import { Observable } from 'rxjs';
import { BaseUrlService } from './base-url.service';

@Injectable()
export class APIClient implements HttpInterceptor {
      private baseUrlService = inject(BaseUrlService);
      
  intercept(req: HttpRequest<any>, next: HttpHandler): Observable<HttpEvent<any>> {
    const apiReq = req.clone({ url: `${this.baseUrlService.getApiBaseUrl()}${req.url}` });
    return next.handle(apiReq);
  }
}
