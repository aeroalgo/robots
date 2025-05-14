using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000B2 RID: 178
	[HandlerCategory("vvPosClose"), HandlerName("StopLoss ATR")]
	public class StopLossAtr : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x06000675 RID: 1653 RVA: 0x0001DA9C File Offset: 0x0001BC9C
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			double num = ATR.iATR(pos.get_Security(), this.AtrPeriod, barNum);
			return pos.get_EntryPrice() + (pos.get_IsLong() ? (-(num * this.StopLossATR)) : (num * this.StopLossATR));
		}

		// Token: 0x1700023C RID: 572
		[HandlerParameter(true, "200", Min = "10", Max = "300", Step = "10", Name = "Период расчёта ATR")]
		public int AtrPeriod
		{
			// Token: 0x06000673 RID: 1651 RVA: 0x0001DA89 File Offset: 0x0001BC89
			get;
			// Token: 0x06000674 RID: 1652 RVA: 0x0001DA91 File Offset: 0x0001BC91
			set;
		}

		// Token: 0x1700023D RID: 573
		public IContext Context
		{
			// Token: 0x06000676 RID: 1654 RVA: 0x0001DAED File Offset: 0x0001BCED
			get;
			// Token: 0x06000677 RID: 1655 RVA: 0x0001DAF5 File Offset: 0x0001BCF5
			set;
		}

		// Token: 0x1700023B RID: 571
		[HandlerParameter(true, "1", Min = "0.05", Max = "2", Step = "0.05", Name = "ATR Стоп-лосс")]
		public double StopLossATR
		{
			// Token: 0x06000671 RID: 1649 RVA: 0x0001DA78 File Offset: 0x0001BC78
			get;
			// Token: 0x06000672 RID: 1650 RVA: 0x0001DA80 File Offset: 0x0001BC80
			set;
		}
	}
}
