using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000099 RID: 153
	[HandlerCategory("vvPosClose"), HandlerName("TrailStopStd Abs CP")]
	public class TrailStopAbsCP : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x0600058A RID: 1418 RVA: 0x0001B990 File Offset: 0x00019B90
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			double num = pos.OpenMFE(barNum);
			double entryPrice = pos.get_EntryPrice();
			if (this.UseCalcPrice)
			{
				double num2 = entryPrice;
				double num3 = CalcEntryPrice.EntryPrice(pos, barNum);
				double num4 = (num2 - num3) * (double)(pos.get_IsLong() ? 1 : -1);
				num += num4;
			}
			double num5 = -this.StopLoss;
			double num6 = pos.get_EntryPrice() + (pos.get_IsLong() ? num5 : (-num5));
			double val = num6;
			if (num > this.TrailEnable)
			{
				num5 = num - this.TrailLoss;
				num6 = pos.get_EntryPrice() + (pos.get_IsLong() ? num5 : (-num5));
			}
			num6 = (pos.get_IsLong() ? Math.Max(num6, val) : Math.Min(num6, val));
			double stop = pos.GetStop(barNum);
			if (stop == 0.0)
			{
				return num6;
			}
			if (!pos.get_IsLong())
			{
				return Math.Min(num6, stop);
			}
			return Math.Max(num6, stop);
		}

		// Token: 0x170001DD RID: 477
		[HandlerParameter(true, "150", Min = "10", Max = "600", Step = "10")]
		public double StopLoss
		{
			// Token: 0x06000582 RID: 1410 RVA: 0x0001B94A File Offset: 0x00019B4A
			get;
			// Token: 0x06000583 RID: 1411 RVA: 0x0001B952 File Offset: 0x00019B52
			set;
		}

		// Token: 0x170001DE RID: 478
		[HandlerParameter(true, "50", Min = "10", Max = "600", Step = "10")]
		public double TrailEnable
		{
			// Token: 0x06000584 RID: 1412 RVA: 0x0001B95B File Offset: 0x00019B5B
			get;
			// Token: 0x06000585 RID: 1413 RVA: 0x0001B963 File Offset: 0x00019B63
			set;
		}

		// Token: 0x170001DF RID: 479
		[HandlerParameter(true, "50", Min = "10", Max = "800", Step = "10")]
		public double TrailLoss
		{
			// Token: 0x06000586 RID: 1414 RVA: 0x0001B96C File Offset: 0x00019B6C
			get;
			// Token: 0x06000587 RID: 1415 RVA: 0x0001B974 File Offset: 0x00019B74
			set;
		}

		// Token: 0x170001E0 RID: 480
		[HandlerParameter(true, "false", NotOptimized = true, Name = "Use Calc Price")]
		public bool UseCalcPrice
		{
			// Token: 0x06000588 RID: 1416 RVA: 0x0001B97D File Offset: 0x00019B7D
			get;
			// Token: 0x06000589 RID: 1417 RVA: 0x0001B985 File Offset: 0x00019B85
			set;
		}
	}
}
