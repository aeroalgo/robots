using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000098 RID: 152
	[HandlerCategory("vvPosClose"), HandlerName("TrailStopStd Abs")]
	public class TrailStopAbs : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x06000580 RID: 1408 RVA: 0x0001B890 File Offset: 0x00019A90
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
				num2 = num - this.TrailLoss;
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

		// Token: 0x170001DA RID: 474
		[HandlerParameter(true, "150", Min = "10", Max = "600", Step = "10")]
		public double StopLoss
		{
			// Token: 0x0600057A RID: 1402 RVA: 0x0001B85D File Offset: 0x00019A5D
			get;
			// Token: 0x0600057B RID: 1403 RVA: 0x0001B865 File Offset: 0x00019A65
			set;
		}

		// Token: 0x170001DB RID: 475
		[HandlerParameter(true, "50", Min = "10", Max = "600", Step = "10")]
		public double TrailEnable
		{
			// Token: 0x0600057C RID: 1404 RVA: 0x0001B86E File Offset: 0x00019A6E
			get;
			// Token: 0x0600057D RID: 1405 RVA: 0x0001B876 File Offset: 0x00019A76
			set;
		}

		// Token: 0x170001DC RID: 476
		[HandlerParameter(true, "50", Min = "10", Max = "800", Step = "10")]
		public double TrailLoss
		{
			// Token: 0x0600057E RID: 1406 RVA: 0x0001B87F File Offset: 0x00019A7F
			get;
			// Token: 0x0600057F RID: 1407 RVA: 0x0001B887 File Offset: 0x00019A87
			set;
		}
	}
}
