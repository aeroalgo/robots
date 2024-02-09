using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000AD RID: 173
	[HandlerCategory("vvPosClose"), HandlerName("Безубыток(%)")]
	public class NoLossPct : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x06000657 RID: 1623 RVA: 0x0001D794 File Offset: 0x0001B994
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			double num = pos.OpenMFEPct(barNum);
			double result = (double)(pos.get_IsLong() ? 0 : 1000000);
			if (num > this.Profit)
			{
				result = pos.get_EntryPrice() * (1.0 + (pos.get_IsLong() ? (this.NoLoss / 100.0) : (-(this.NoLoss / 100.0))));
			}
			return result;
		}

		// Token: 0x17000232 RID: 562
		[HandlerParameter(true, "0.1", Min = "0.05", Max = "0.2", Step = "0.05", Name = "Безубыток")]
		public double NoLoss
		{
			// Token: 0x06000655 RID: 1621 RVA: 0x0001D782 File Offset: 0x0001B982
			get;
			// Token: 0x06000656 RID: 1622 RVA: 0x0001D78A File Offset: 0x0001B98A
			set;
		}

		// Token: 0x17000231 RID: 561
		[HandlerParameter(true, "0.3", Min = "0.2", Max = "1", Step = "0.05")]
		public double Profit
		{
			// Token: 0x06000653 RID: 1619 RVA: 0x0001D771 File Offset: 0x0001B971
			get;
			// Token: 0x06000654 RID: 1620 RVA: 0x0001D779 File Offset: 0x0001B979
			set;
		}
	}
}
