using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000A0 RID: 160
	[HandlerCategory("vvPosClose"), HandlerName("Трейл по теням (AER)")]
	public class TrailOnShadowsAER : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs, IContextUses
	{
		// Token: 0x060005CA RID: 1482 RVA: 0x0001C43C File Offset: 0x0001A63C
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			bool eRPeriod = this.ERPeriod != 0;
			pos.OpenMFEPct(barNum);
			ISecurity security = pos.get_Security();
			int num = this.BarsMax;
			if (eRPeriod)
			{
				double num2 = KER.iKER(security.get_ClosePrices(), barNum, this.ERPeriod);
				num = Convert.ToInt32((double)this.BarsMax + num2 * (double)(this.BarsMin - this.BarsMax));
			}
			num = Math.Min(num, barNum);
			double num3 = pos.get_IsLong() ? vvSeries.iLowest(security.get_LowPrices(), barNum, num) : vvSeries.iHighest(security.get_HighPrices(), barNum, num);
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

		// Token: 0x170001F7 RID: 503
		[HandlerParameter(true, "15", Min = "1", Max = "20", Step = "1")]
		public int BarsMax
		{
			// Token: 0x060005C4 RID: 1476 RVA: 0x0001C409 File Offset: 0x0001A609
			get;
			// Token: 0x060005C5 RID: 1477 RVA: 0x0001C411 File Offset: 0x0001A611
			set;
		}

		// Token: 0x170001F8 RID: 504
		[HandlerParameter(true, "3", Min = "1", Max = "20", Step = "1")]
		public int BarsMin
		{
			// Token: 0x060005C6 RID: 1478 RVA: 0x0001C41A File Offset: 0x0001A61A
			get;
			// Token: 0x060005C7 RID: 1479 RVA: 0x0001C422 File Offset: 0x0001A622
			set;
		}

		// Token: 0x170001FA RID: 506
		public IContext Context
		{
			// Token: 0x060005CB RID: 1483 RVA: 0x0001C51C File Offset: 0x0001A71C
			get;
			// Token: 0x060005CC RID: 1484 RVA: 0x0001C524 File Offset: 0x0001A724
			set;
		}

		// Token: 0x170001F9 RID: 505
		[HandlerParameter(true, "0", Min = "1", Max = "20", Step = "1")]
		public int ERPeriod
		{
			// Token: 0x060005C8 RID: 1480 RVA: 0x0001C42B File Offset: 0x0001A62B
			get;
			// Token: 0x060005C9 RID: 1481 RVA: 0x0001C433 File Offset: 0x0001A633
			set;
		}
	}
}
