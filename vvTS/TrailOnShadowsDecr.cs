using System;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200009F RID: 159
	[HandlerCategory("vvPosClose"), HandlerName("Трейл по теням Decre")]
	public class TrailOnShadowsDecr : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs, IContextUses
	{
		// Token: 0x060005C0 RID: 1472 RVA: 0x0001C2CC File Offset: 0x0001A4CC
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			ISecurity security = pos.get_Security();
			int num = barNum - pos.get_EntryBarNum();
			int num2 = Math.Min(this.Bars_TrailStopStart, barNum);
			double num3 = pos.get_IsLong() ? Indicators.Lowest(security.get_LowPrices(), barNum, Math.Max(num2 - num, 1)) : Indicators.Highest(security.get_HighPrices(), barNum, Math.Max(num2 - num, 1));
			if (this.MaxStopLoss > 0.0 && (pos.get_IsLong() ? (pos.get_EntryPrice() > num3) : (pos.get_EntryPrice() < num3)))
			{
				if (pos.get_IsLong() && pos.get_EntryPrice() - num3 > this.MaxStopLoss)
				{
					num3 = pos.get_EntryPrice() - this.MaxStopLoss;
				}
				if (pos.get_IsShort() && num3 - pos.get_EntryPrice() > this.MaxStopLoss)
				{
					num3 = pos.get_EntryPrice() + this.MaxStopLoss;
				}
			}
			double stop = pos.GetStop(barNum);
			if (stop == 0.0)
			{
				return num3;
			}
			if (!pos.get_IsLong())
			{
				return Math.Min(num3, stop);
			}
			return Math.Max(num3, stop);
		}

		// Token: 0x170001F4 RID: 500
		[HandlerParameter(true, "10", Min = "1", Max = "20", Step = "1", Name = "Баров - ТрейлСтоп")]
		public int Bars_TrailStopStart
		{
			// Token: 0x060005BC RID: 1468 RVA: 0x0001C2A8 File Offset: 0x0001A4A8
			get;
			// Token: 0x060005BD RID: 1469 RVA: 0x0001C2B0 File Offset: 0x0001A4B0
			set;
		}

		// Token: 0x170001F6 RID: 502
		public IContext Context
		{
			// Token: 0x060005C1 RID: 1473 RVA: 0x0001C3F0 File Offset: 0x0001A5F0
			get;
			// Token: 0x060005C2 RID: 1474 RVA: 0x0001C3F8 File Offset: 0x0001A5F8
			set;
		}

		// Token: 0x170001F5 RID: 501
		[HandlerParameter(true, "0", Min = "0", Max = "500", Step = "1", Name = "Макс. стоплосс (п)")]
		public double MaxStopLoss
		{
			// Token: 0x060005BE RID: 1470 RVA: 0x0001C2B9 File Offset: 0x0001A4B9
			get;
			// Token: 0x060005BF RID: 1471 RVA: 0x0001C2C1 File Offset: 0x0001A4C1
			set;
		}
	}
}
