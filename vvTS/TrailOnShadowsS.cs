using System;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200009E RID: 158
	[HandlerCategory("vvPosClose"), HandlerName("Трейл по теням Simple")]
	public class TrailOnShadowsS : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs, IContextUses
	{
		// Token: 0x060005B8 RID: 1464 RVA: 0x0001C180 File Offset: 0x0001A380
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			pos.OpenMFEPct(barNum);
			ISecurity security = pos.get_Security();
			int num = Math.Min(this.Bars_TrailStop, barNum);
			double num2 = pos.get_IsLong() ? Indicators.Lowest(security.get_LowPrices(), barNum, num) : Indicators.Highest(security.get_HighPrices(), barNum, num);
			if (this.MaxStopLoss > 0.0 && (pos.get_IsLong() ? (pos.get_EntryPrice() > num2) : (pos.get_EntryPrice() < num2)))
			{
				if (pos.get_IsLong() && pos.get_EntryPrice() - num2 > this.MaxStopLoss)
				{
					num2 = pos.get_EntryPrice() - this.MaxStopLoss;
				}
				if (pos.get_IsShort() && num2 - pos.get_EntryPrice() > this.MaxStopLoss)
				{
					num2 = pos.get_EntryPrice() + this.MaxStopLoss;
				}
			}
			double stop = pos.GetStop(barNum);
			if (stop == 0.0)
			{
				return num2;
			}
			if (!pos.get_IsLong())
			{
				return Math.Min(num2, stop);
			}
			return Math.Max(num2, stop);
		}

		// Token: 0x170001F1 RID: 497
		[HandlerParameter(true, "15", Min = "1", Max = "20", Step = "1", Name = "Баров - ТрейлСтоп")]
		public int Bars_TrailStop
		{
			// Token: 0x060005B4 RID: 1460 RVA: 0x0001C15D File Offset: 0x0001A35D
			get;
			// Token: 0x060005B5 RID: 1461 RVA: 0x0001C165 File Offset: 0x0001A365
			set;
		}

		// Token: 0x170001F3 RID: 499
		public IContext Context
		{
			// Token: 0x060005B9 RID: 1465 RVA: 0x0001C28F File Offset: 0x0001A48F
			get;
			// Token: 0x060005BA RID: 1466 RVA: 0x0001C297 File Offset: 0x0001A497
			set;
		}

		// Token: 0x170001F2 RID: 498
		[HandlerParameter(true, "0", Min = "0", Max = "500", Step = "1", Name = "Макс. стоплосс (п)")]
		public double MaxStopLoss
		{
			// Token: 0x060005B6 RID: 1462 RVA: 0x0001C16E File Offset: 0x0001A36E
			get;
			// Token: 0x060005B7 RID: 1463 RVA: 0x0001C176 File Offset: 0x0001A376
			set;
		}
	}
}
