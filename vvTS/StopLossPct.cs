using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000B1 RID: 177
	[HandlerCategory("vvPosClose"), HandlerName("StopLoss Pct")]
	public class StopLossPct : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x0600066F RID: 1647 RVA: 0x0001DA10 File Offset: 0x0001BC10
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			pos.OpenMFEPct(barNum);
			return pos.get_EntryPrice() * (1.0 + (pos.get_IsLong() ? (-(this.StopLoss / 100.0)) : (this.StopLoss / 100.0)));
		}

		// Token: 0x1700023A RID: 570
		[HandlerParameter(true, "0.2", Min = "0.05", Max = "0.6", Step = "0.05", Name = "Стоп-лосс")]
		public double StopLoss
		{
			// Token: 0x0600066D RID: 1645 RVA: 0x0001D9FE File Offset: 0x0001BBFE
			get;
			// Token: 0x0600066E RID: 1646 RVA: 0x0001DA06 File Offset: 0x0001BC06
			set;
		}
	}
}
