using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000B4 RID: 180
	[HandlerCategory("vvPosClose"), HandlerName("TakeProfit Pct")]
	public class TakeProfitPct : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x0600067F RID: 1663 RVA: 0x0001DB74 File Offset: 0x0001BD74
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			pos.OpenMFEPct(barNum);
			return pos.get_EntryPrice() * (1.0 + (pos.get_IsLong() ? (this.TakeProfit / 100.0) : (-(this.TakeProfit / 100.0))));
		}

		// Token: 0x1700023F RID: 575
		[HandlerParameter(true, "0.4", Min = "0.05", Max = "0.6", Step = "0.05")]
		public double TakeProfit
		{
			// Token: 0x0600067D RID: 1661 RVA: 0x0001DB62 File Offset: 0x0001BD62
			get;
			// Token: 0x0600067E RID: 1662 RVA: 0x0001DB6A File Offset: 0x0001BD6A
			set;
		}
	}
}
