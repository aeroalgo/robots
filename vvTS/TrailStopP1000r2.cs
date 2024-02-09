using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000AB RID: 171
	[HandlerCategory("vvPosClose"), HandlerName("TrailStop P1000R-2")]
	public class TrailStopP1000r2 : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x0600064B RID: 1611 RVA: 0x0001D554 File Offset: 0x0001B754
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			double num = pos.OpenMFEPct(barNum);
			double num2 = (0.0 - this.StopLoss) / 100.0;
			double num3 = pos.get_EntryPrice() * (1.0 + (pos.get_IsLong() ? num2 : (-num2)));
			double val = num3;
			if (num > this.TrailEnable)
			{
				if (this.ProfitLimitMax != 0.0 && this.MaxProfitTrLoss != 0.0)
				{
					num2 = ((num > this.ProfitLimitMax) ? ((num - this.MaxProfitTrLoss) / 100.0) : ((num - this.TrailLoss) / 100.0));
				}
				else if (this.ProfitLimitMin != 0.0 && this.MinProfitTrLoss != 0.0)
				{
					num2 = ((num > this.ProfitLimitMin) ? ((num - this.MinProfitTrLoss) / 100.0) : ((num - this.TrailLoss) / 100.0));
				}
				else
				{
					num2 = (num - this.TrailLoss) / 100.0;
				}
				num3 = pos.get_EntryPrice() * (1.0 + (pos.get_IsLong() ? num2 : (-num2)));
			}
			num3 = (pos.get_IsLong() ? Math.Max(num3, val) : Math.Min(num3, val));
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

		// Token: 0x1700022E RID: 558
		[HandlerParameter(true, "0.1", Min = "0.1", Max = "0.6", Step = "0.05", Name = "MaxProfitTrLoss")]
		public double MaxProfitTrLoss
		{
			// Token: 0x06000649 RID: 1609 RVA: 0x0001D543 File Offset: 0x0001B743
			get;
			// Token: 0x0600064A RID: 1610 RVA: 0x0001D54B File Offset: 0x0001B74B
			set;
		}

		// Token: 0x1700022D RID: 557
		[HandlerParameter(true, "0.3", Min = "0.1", Max = "0.6", Step = "0.05", Name = "MinProfitTrLoss")]
		public double MinProfitTrLoss
		{
			// Token: 0x06000647 RID: 1607 RVA: 0x0001D532 File Offset: 0x0001B732
			get;
			// Token: 0x06000648 RID: 1608 RVA: 0x0001D53A File Offset: 0x0001B73A
			set;
		}

		// Token: 0x1700022A RID: 554
		[HandlerParameter(true, "0", Min = "0", Max = "0.5", Step = "0.05", Name = "ProfitLimitMax")]
		public double ProfitLimitMax
		{
			// Token: 0x06000641 RID: 1601 RVA: 0x0001D4FF File Offset: 0x0001B6FF
			get;
			// Token: 0x06000642 RID: 1602 RVA: 0x0001D507 File Offset: 0x0001B707
			set;
		}

		// Token: 0x17000229 RID: 553
		[HandlerParameter(true, "0", Min = "0", Max = "0.5", Step = "0.05", Name = "ProfitLimitMin")]
		public double ProfitLimitMin
		{
			// Token: 0x0600063F RID: 1599 RVA: 0x0001D4EE File Offset: 0x0001B6EE
			get;
			// Token: 0x06000640 RID: 1600 RVA: 0x0001D4F6 File Offset: 0x0001B6F6
			set;
		}

		// Token: 0x17000228 RID: 552
		[HandlerParameter(true, "1.5", Min = "0.1", Max = "0.5", Step = "0.05", Name = "StopLoss")]
		public double StopLoss
		{
			// Token: 0x0600063D RID: 1597 RVA: 0x0001D4DD File Offset: 0x0001B6DD
			get;
			// Token: 0x0600063E RID: 1598 RVA: 0x0001D4E5 File Offset: 0x0001B6E5
			set;
		}

		// Token: 0x1700022B RID: 555
		[HandlerParameter(true, "0.5", Min = "0.1", Max = "1", Step = "0.05", Name = "TrailEnable")]
		public double TrailEnable
		{
			// Token: 0x06000643 RID: 1603 RVA: 0x0001D510 File Offset: 0x0001B710
			get;
			// Token: 0x06000644 RID: 1604 RVA: 0x0001D518 File Offset: 0x0001B718
			set;
		}

		// Token: 0x1700022C RID: 556
		[HandlerParameter(true, "0.5", Min = "0.1", Max = "1", Step = "0.05", Name = "TrailLoss")]
		public double TrailLoss
		{
			// Token: 0x06000645 RID: 1605 RVA: 0x0001D521 File Offset: 0x0001B721
			get;
			// Token: 0x06000646 RID: 1606 RVA: 0x0001D529 File Offset: 0x0001B729
			set;
		}
	}
}
