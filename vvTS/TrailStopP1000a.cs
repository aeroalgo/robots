using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000A8 RID: 168
	[HandlerCategory("vvPosClose"), HandlerName("TrailStop P1000A")]
	public class TrailStopP1000a : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x0600061E RID: 1566 RVA: 0x0001D0BC File Offset: 0x0001B2BC
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			double num = pos.OpenMFE(barNum);
			double num2 = -this.StopLoss;
			double num3 = pos.get_EntryPrice() + (pos.get_IsLong() ? num2 : (-num2));
			double val = num3;
			if (num > this.TrailEnable)
			{
				if (this.ProfitLimit != 0.0 && this.TrailLoss2 != 0.0)
				{
					num2 = ((num > this.ProfitLimit) ? (num - this.TrailLoss2) : (num - this.TrailLoss));
				}
				else
				{
					num2 = num - this.TrailLoss;
				}
				num3 = pos.get_EntryPrice() + (pos.get_IsLong() ? num2 : (-num2));
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

		// Token: 0x17000218 RID: 536
		[HandlerParameter(true, "1000", Min = "100", Max = "1000", Step = "10", Name = "ProfitLimit")]
		public double ProfitLimit
		{
			// Token: 0x06000616 RID: 1558 RVA: 0x0001D076 File Offset: 0x0001B276
			get;
			// Token: 0x06000617 RID: 1559 RVA: 0x0001D07E File Offset: 0x0001B27E
			set;
		}

		// Token: 0x17000217 RID: 535
		[HandlerParameter(true, "150", Min = "10", Max = "600", Step = "10", Name = "StopLoss")]
		public double StopLoss
		{
			// Token: 0x06000614 RID: 1556 RVA: 0x0001D065 File Offset: 0x0001B265
			get;
			// Token: 0x06000615 RID: 1557 RVA: 0x0001D06D File Offset: 0x0001B26D
			set;
		}

		// Token: 0x17000219 RID: 537
		[HandlerParameter(true, "50", Min = "10", Max = "600", Step = "10", Name = "TrailEnable")]
		public double TrailEnable
		{
			// Token: 0x06000618 RID: 1560 RVA: 0x0001D087 File Offset: 0x0001B287
			get;
			// Token: 0x06000619 RID: 1561 RVA: 0x0001D08F File Offset: 0x0001B28F
			set;
		}

		// Token: 0x1700021A RID: 538
		[HandlerParameter(true, "100", Min = "10", Max = "600", Step = "10", Name = "TrailLoss")]
		public double TrailLoss
		{
			// Token: 0x0600061A RID: 1562 RVA: 0x0001D098 File Offset: 0x0001B298
			get;
			// Token: 0x0600061B RID: 1563 RVA: 0x0001D0A0 File Offset: 0x0001B2A0
			set;
		}

		// Token: 0x1700021B RID: 539
		[HandlerParameter(true, "50", Min = "10", Max = "600", Step = "10", Name = "TrLoss2")]
		public double TrailLoss2
		{
			// Token: 0x0600061C RID: 1564 RVA: 0x0001D0A9 File Offset: 0x0001B2A9
			get;
			// Token: 0x0600061D RID: 1565 RVA: 0x0001D0B1 File Offset: 0x0001B2B1
			set;
		}
	}
}
