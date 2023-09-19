export interface Device {
  uid: string;
  name: string;
  mac: string;
  ip: string;
}

export enum DeviceStatus {
  Online = 'Online',
  Offline = 'Offline',
}

export interface HostInterface {
  name: string;
  mac: string;
  ips: string[];
  chosen: boolean;
}
