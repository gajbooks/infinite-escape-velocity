import { Component, ElementRef, EventEmitter, HostListener, Input, ViewChild } from '@angular/core';
import { ClientServerMessage } from 'bindings/ClientServerMessage';
import { ControlInput } from 'bindings/ControlInput';
import { DynamicObjectCreationData } from 'bindings/DynamicObjectCreationData';
import { DynamicObjectDestructionData } from 'bindings/DynamicObjectDestructionData';
import { DynamicObjectUpdateData } from 'bindings/DynamicObjectUpdateData';
import { ServerClientMessage } from 'bindings/ServerClientMessage';
import { ViewportFollowData } from 'bindings/ViewportFollowData';
import Konva from 'konva';
import { Subject, interval } from 'rxjs';
import { ENVIRONMENT } from 'src/environments/environment';
import { StarfieldGenerator } from './starfield-generator';
import { AssetIndexValue } from 'bindings/AssetIndexValue';

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

class RenderedObject {
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
  @Input({ required: true }) assetIndexList!: Promise<AssetIndexValue[]>;
  @ViewChild('gameWindow') gameWindow!: ElementRef;

  renderer!: Konva.Stage;
  shipLayer!: Konva.Layer;
  planetoidLayer!: Konva.Layer;
  renderLoop = interval(16);
  assetCache: Map<BigInt, HTMLImageElement> = new Map();
  assetIdToName: Map<BigInt, string> = new Map();
  dynamicObjects: Map<BigInt, RenderedObject> = new Map();
  camera_center_x: number = 0.0;
  camera_center_y: number = 0.0;
  camera_center_entity: BigInt | null = null;
  key_status: Map<String, KeyStatus> = new Map();
  starfield_renderer!: StarfieldGenerator;

  constructor() {
  }

  object_offset_x(): number {
    return (this.renderer.width() / 2) - this.camera_center_x;
  }

  object_offset_y(): number {
    return (this.renderer.height() / 2) - this.camera_center_y;
  }

  refreshScreen() {
    if (this.camera_center_entity != null) {
      let center_entity = this.dynamicObjects.get(this.camera_center_entity);
      if (typeof center_entity !== 'undefined') {
        this.camera_center_x = center_entity.x;
        this.camera_center_y = center_entity.y;
      }
    }

    this.starfield_renderer.draw_stars(this.camera_center_x, this.camera_center_y, this.renderer.width(), this.renderer.height());

    this.dynamicObjects.forEach((val) => {
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
      }
      if (this.renderer.height() != this.gameWindow.nativeElement.clientHeight) {
        this.renderer.height(this.gameWindow.nativeElement.clientHeight);
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

    let star_layer = new Konva.Layer({ listening: false });
    this.renderer.add(star_layer);
    this.starfield_renderer = new StarfieldGenerator(star_layer);

    this.planetoidLayer = new Konva.Layer();
    this.renderer.add(this.planetoidLayer);

    this.shipLayer = new Konva.Layer();
    this.renderer.add(this.shipLayer);

    this.renderLoop.subscribe(() => {
      this.refreshScreen();
    });

    let assetIdToName = this.assetIdToName;

    this.assetIndexList.then(index => {
      index.forEach((asset, _ignored) => {
        assetIdToName.set(asset.id, asset.name);
      });
    });


    let canvas = this;
    this.incomingMessages.subscribe({
      next(val) {
        if (val.type == 'DynamicObjectCreation') {
          let new_dynamic_object = val.data as DynamicObjectCreationData;

          if (canvas.dynamicObjects.has(new_dynamic_object.id) == false) {
            if (canvas.assetCache.has(new_dynamic_object.object_asset) == false) {
              let index_has = canvas.assetIdToName.get(new_dynamic_object.object_asset);
              if (typeof index_has !== 'undefined') {
                let new_image = new Image();
                new_image.src = `${ENVIRONMENT.GAME_SERVER_URL}/assets/name/${index_has}`;
                canvas.assetCache.set(new_dynamic_object.object_asset, new_image);
              } else {
                console.error(`Tried to use missing asset Id ${new_dynamic_object.object_asset}`);
              }
            }

            let dynamic_object_image = canvas.assetCache.get(new_dynamic_object.object_asset);

            if (typeof dynamic_object_image !== 'undefined') {
              let konva_image = new Konva.Image({
                image: dynamic_object_image,
                width: new_dynamic_object.display_radius,
                height: new_dynamic_object.display_radius,
                visible: false,
              });

              let setAttributes = () => {
                konva_image.setAttrs({
                  offsetX: konva_image.width() / 2,
                  offsetY: konva_image.height() / 2,
                })
              };

              let loadedEventEmitter: Subject<null>;
              if (typeof (dynamic_object_image as any).onLoadEventHandler === 'undefined') {
                loadedEventEmitter = new Subject();
                (dynamic_object_image as any).onLoadEventHandler = loadedEventEmitter;
                dynamic_object_image.onload = () => {
                  loadedEventEmitter.next(null);
                }
              } else {
                loadedEventEmitter = (dynamic_object_image as any).onLoadEventHandler;
              }

              loadedEventEmitter.subscribe(setAttributes);

              setAttributes();

              canvas.dynamicObjects.set(new_dynamic_object.id, new RenderedObject(
                -10000.0,
                -10000.0,
                0.0,
                konva_image
              ));

              let rendered_object = <RenderedObject>canvas.dynamicObjects.get(new_dynamic_object.id);

              switch (new_dynamic_object.view_layer) {
                case 'Background':
                  break;
                case 'Planetoids':
                  canvas.planetoidLayer.add(rendered_object.graphics);
                  break;
                case 'Ships':
                  canvas.shipLayer.add(rendered_object.graphics);
                  break;
                case 'Weapons':
                  break;
              };
            }
          }
        }

        else if (val.type == 'DynamicObjectUpdate') {
          let updated_object = val.data as DynamicObjectUpdateData;

          // Correct rotational coordinates from radians to degrees for Konva rendering
          if (updated_object.rotation != null) {
            updated_object.rotation.rotation = updated_object.rotation.rotation * (180 / Math.PI);
          }

          let dynamic_object = canvas.dynamicObjects.get(updated_object.id);

          if (typeof dynamic_object !== 'undefined') {
            dynamic_object.x = updated_object.x;
            dynamic_object.y = updated_object.y;
            dynamic_object.rotation = updated_object.rotation?.rotation ?? 0.0;

            dynamic_object.graphics.show();
          }
        }

        else if (val.type == 'DynamicObjectDestruction') {
          let deleted_object = val.data as DynamicObjectDestructionData;
          let deleted_dynamic_object = canvas.dynamicObjects.get(deleted_object.id);

          if (typeof deleted_dynamic_object !== 'undefined') {
            deleted_dynamic_object.graphics.remove();
            canvas.dynamicObjects.delete(deleted_object.id);
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
      this.outgoingMessages.next({ type: 'ControlInput', input: input, pressed: pressed });
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
