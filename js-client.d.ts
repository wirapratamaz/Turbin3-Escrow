declare module '../dist/js-client' {
  export const ESCROW_PROGRAM_ADDRESS: string;
  export function getMakeOfferInstructionAsync(params: any): Promise<any>;
  export function getTakeOfferInstructionAsync(params: any): Promise<any>;
  export function getRefundOfferInstructionAsync(params: any): Promise<any>;
} 