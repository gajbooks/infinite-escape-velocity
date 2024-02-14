import { group } from '@angular/animations';
import { Component, ElementRef, HostListener, Input, ViewChild } from '@angular/core';
import { ClientServerMessage } from 'bindings/ClientServerMessage';
import { ControlInput } from 'bindings/ControlInput';
import { DynamicObjectCreationData } from 'bindings/DynamicObjectCreationData';
import { DynamicObjectDestructionData } from 'bindings/DynamicObjectDestructionData';
import { DynamicObjectMessageData } from 'bindings/DynamicObjectMessageData';
import { ServerClientMessage } from 'bindings/ServerClientMessage';
import { ViewportFollowData } from 'bindings/ViewportFollowData';
import Konva from 'konva';
import { Subject, interval } from 'rxjs';
import { ENVIRONMENT } from 'src/environments/environment';
import { StarfieldGenerator } from './starfield-generator';

type SendMessageFunction = (input: ControlInput, pressed: boolean) => void;

class KeyStatus {
  pressed: boolean = false;
  input: ControlInput;
  send_message: SendMessageFunction;
  
  constructor(input: ControlInput, send: SendMessageFunction) {
    this.input = input;
    this.send_message = send;
  }

  updateStatus(key_down: boolean) {
    if (this.pressed == false) {
      if (key_down == true) {
        this.pressed = true;
        this.send_message(this.input, this.pressed);
      }
    } else {
      if (key_down == false) {
        this.pressed = false;
        this.send_message(this.input, this.pressed);
      }
    }
  }
}

class RenderShip {
  x: number;
  y: number;
  rotation: number;
  graphics: Konva.Image

  constructor(x: number, y: number, rotation: number, graphics: Konva.Image) {
    this.x = x;
    this.y = y;
    this.rotation = rotation;
    this.graphics = graphics;
  }
}

@Component({
  selector: 'app-gameplay-canvas',
  standalone: true,
  imports: [],
  templateUrl: './gameplay-canvas.component.html',
  styleUrl: './gameplay-canvas.component.less'
})
export class GameplayCanvasComponent {
  @Input({ required: true }) incomingMessages!: Subject<ServerClientMessage>;
  @Input({ required: true }) outgoingMessages!: Subject<ClientServerMessage>;
  @ViewChild('gameWindow') gameWindow!: ElementRef;

  renderer!: Konva.Stage;
  shipLayer!: Konva.Layer;
  renderLoop = interval(16);
  shipImage = new Image();
  ships: Map<BigInt, RenderShip> = new Map();
  half_screen_width: number = 0.0;
  half_screen_height: number = 0.0;
  camera_center_x: number = 0.0;
  camera_center_y: number = 0.0;
  camera_center_entity: BigInt | null = null;
  key_status: Map<String, KeyStatus> = new Map();
  starfield_renderer!: StarfieldGenerator;

  constructor() {
    this.shipImage.src = ENVIRONMENT.GAME_SERVER_URL + '/assets/name/default_image';
  }

  object_offset_x(): number {
    return this.half_screen_width - this.camera_center_x;
  }

  object_offset_y(): number {
    return this.half_screen_height - this.camera_center_y;
  }

  refreshScreen() {
    if (this.camera_center_entity != null) {
      let center_entity = this.ships.get(this.camera_center_entity);
      if (typeof center_entity !== 'undefined') {
        this.camera_center_x = center_entity.x;
        this.camera_center_y = center_entity.y;
      }
    }

    this.starfield_renderer.draw_stars(this.camera_center_x, this.camera_center_y, this.renderer.width(), this.renderer.height());

    this.ships.forEach((val) => {
      val.graphics.x(val.x + this.object_offset_x());
      val.graphics.y(val.y + this.object_offset_y());
      val.graphics.rotation(val.rotation);
    });

    this.renderer.draw();
  }

  resizeRenderer() {
    if (this.renderer != null && this.gameWindow != null) {
      if (this.renderer.width() != this.gameWindow.nativeElement.clientWidth) {
        this.renderer.width(this.gameWindow.nativeElement.clientWidth);
        this.half_screen_width = this.renderer.width() / 2;
      }
      if (this.renderer.height() != this.gameWindow.nativeElement.clientHeight) {
        this.renderer.height(this.gameWindow.nativeElement.clientHeight);
        this.half_screen_height = this.renderer.height() / 2;
      }
    }
  }

  @HostListener('document:keydown', ['$event'])
  @HostListener('document:keyup', ['$event'])
  onKeyChange(event: KeyboardEvent) {
    let pressed = event.type == 'keydown' ? true : false;
    let button = this.key_status.get(event.key);
    if (typeof button !== 'undefined') {
      button.updateStatus(pressed);
    }
  }

  ngOnInit() {
    this.renderer = new Konva.Stage({
      container: 'gameWindow',
      width: 1000,
      height: 1000
    });

    let star_layer = new Konva.Layer({listening: false});
    this.renderer.add(star_layer);
    this.starfield_renderer = new StarfieldGenerator(star_layer);

    this.shipLayer = new Konva.Layer();
    this.renderer.add(this.shipLayer);
    this.renderLoop.subscribe(() => {
      this.refreshScreen();
    });

    let canvas = this;
    this.incomingMessages.subscribe({
      next(val) {
        if (val.type == 'DynamicObjectCreation') {
          let new_ship = val.data as DynamicObjectCreationData;
          // We don't care about imaginary ships right now
        }

        else if (val.type == 'DynamicObjectUpdate') {
          let updated_ship = val.data as DynamicObjectMessageData;

          // Correct rotational coordinates from radians to degrees for Konva rendering
          if (updated_ship.rotation != null) {
            updated_ship.rotation.rotation = updated_ship.rotation.rotation * (180 / Math.PI);
          }

          if (canvas.ships.has(updated_ship.id) == false) {
            canvas.ships.set(updated_ship.id, new RenderShip(
              updated_ship.x,
              updated_ship.y,
              updated_ship.rotation?.rotation ?? 0.0,
              new Konva.Image({
                image: canvas.shipImage,
                offsetX: canvas.shipImage.width / 2,
                offsetY: canvas.shipImage.height / 2,
                x: updated_ship.x + canvas.object_offset_x(),
                y: updated_ship.y + canvas.object_offset_y(),
                rotation: updated_ship.rotation?.rotation
              })
            ));
            let ship = <RenderShip>canvas.ships.get(updated_ship.id);
            canvas.shipLayer.add(ship.graphics);
          }

          let ship = canvas.ships.get(updated_ship.id);

          if (typeof ship !== 'undefined') {
            ship.x = updated_ship.x;
            ship.y = updated_ship.y;
            ship.rotation = updated_ship.rotation?.rotation ?? 0;
          }

        }

        else if (val.type == 'DynamicObjectDestruction') {
          let deleted_ship = val.data as DynamicObjectDestructionData;
          let ship = canvas.ships.get(deleted_ship.id);

          if (typeof ship !== 'undefined') {
            ship.graphics.remove();
            canvas.ships.delete(deleted_ship.id);
          }
        }

        else if (val.type == 'ViewportFollow') {
          let viewport_follow = <ViewportFollowData>val.data;
          if (viewport_follow.subtype == 'Disconnected') {
            // Do nothing, leave viewport center in previous position
          } else if (viewport_follow.subtype == 'Entity') {
            canvas.camera_center_entity = viewport_follow.id;
          } else if (viewport_follow.subtype == 'Static') {
            canvas.camera_center_x = viewport_follow.x;
            canvas.camera_center_y = viewport_follow.y;
          }
        }

      }
    })

    let send_message = (input: ControlInput, pressed: boolean) => {
      this.outgoingMessages.next({type: 'ControlInput', input: input, pressed: pressed});
    };

    this.key_status.set('ArrowDown', new KeyStatus('Backward', send_message));
    this.key_status.set('ArrowUp', new KeyStatus('Forward', send_message));
    this.key_status.set('ArrowLeft', new KeyStatus('Left', send_message));
    this.key_status.set('ArrowRight', new KeyStatus('Right', send_message));
    this.key_status.set(' ', new KeyStatus('Fire', send_message));
  }

  ngAfterViewChecked() {
    this.resizeRenderer();
  }
}
