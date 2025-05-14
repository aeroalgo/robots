using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000AE RID: 174
	[HandlerCategory("vvPosClose"), HandlerName("Безубыток (+SL)")]
	public class NoLossAbsSL : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x0600065F RID: 1631 RVA: 0x0001D84C File Offset: 0x0001BA4C
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			double num = pos.OpenMFE(barNum);
			double result = pos.get_EntryPrice() + (pos.get_IsLong() ? (-this.StopLoss) : this.StopLoss);
			if (num > this.Profit)
			{
				result = pos.get_EntryPrice() + (pos.get_IsLong() ? this.NoLoss : (-this.NoLoss));
			}
			return result;
		}

		// Token: 0x17000234 RID: 564
		[HandlerParameter(true, "100", Min = "10", Max = "200", Step = "10", Name = "Безубыток")]
		public double NoLoss
		{
			// Token: 0x0600065B RID: 1627 RVA: 0x0001D828 File Offset: 0x0001BA28
			get;
			// Token: 0x0600065C RID: 1628 RVA: 0x0001D830 File Offset: 0x0001BA30
			set;
		}

		// Token: 0x17000233 RID: 563
		[HandlerParameter(true, "400", Min = "10", Max = "500", Step = "10")]
		public double Profit
		{
			// Token: 0x06000659 RID: 1625 RVA: 0x0001D817 File Offset: 0x0001BA17
			get;
			// Token: 0x0600065A RID: 1626 RVA: 0x0001D81F File Offset: 0x0001BA1F
			set;
		}

		// Token: 0x17000235 RID: 565
		[HandlerParameter(true, "200", Min = "10", Max = "500", Step = "10", Name = "Стоп-лосс")]
		public double StopLoss
		{
			// Token: 0x0600065D RID: 1629 RVA: 0x0001D839 File Offset: 0x0001BA39
			get;
			// Token: 0x0600065E RID: 1630 RVA: 0x0001D841 File Offset: 0x0001BA41
			set;
		}
	}
}
