using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020001A9 RID: 425
	[HandlerCategory("vvAverages"), HandlerName("xMASlope")]
	public class xMASlope : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000D7A RID: 3450 RVA: 0x0003B0F4 File Offset: 0x000392F4
		public IList<double> Execute(IList<double> ma)
		{
			double[] array = new double[ma.Count];
			IList<double> result = array;
			for (int i = this.BarsNumber; i < ma.Count; i++)
			{
				double num = ma[i];
				double num2 = ma[i - this.BarsNumber];
				double num3 = num - num2;
				if (num3 > this.Sense)
				{
					array[i] = num3;
				}
				else if (num3 < -this.Sense)
				{
					array[i] = num3;
				}
				else
				{
					array[i] = 0.0;
				}
			}
			if (this.Smooth > 0)
			{
				result = JMA.GenJMA(array, this.Smooth, this.SmoothPhase);
			}
			return result;
		}

		// Token: 0x1700045D RID: 1117
		[HandlerParameter(true, "1", Min = "1", Max = "10", Step = "1")]
		public int BarsNumber
		{
			// Token: 0x06000D72 RID: 3442 RVA: 0x0003B0AD File Offset: 0x000392AD
			get;
			// Token: 0x06000D73 RID: 3443 RVA: 0x0003B0B5 File Offset: 0x000392B5
			set;
		}

		// Token: 0x17000461 RID: 1121
		public IContext Context
		{
			// Token: 0x06000D7B RID: 3451 RVA: 0x0003B194 File Offset: 0x00039394
			get;
			// Token: 0x06000D7C RID: 3452 RVA: 0x0003B19C File Offset: 0x0003939C
			set;
		}

		// Token: 0x1700045E RID: 1118
		[HandlerParameter(true, "3", Min = "0", Max = "20", Step = "0.1")]
		public double Sense
		{
			// Token: 0x06000D74 RID: 3444 RVA: 0x0003B0BE File Offset: 0x000392BE
			get;
			// Token: 0x06000D75 RID: 3445 RVA: 0x0003B0C6 File Offset: 0x000392C6
			set;
		}

		// Token: 0x1700045F RID: 1119
		[HandlerParameter(true, "0", Min = "1", Max = "20", Step = "1")]
		public int Smooth
		{
			// Token: 0x06000D76 RID: 3446 RVA: 0x0003B0CF File Offset: 0x000392CF
			get;
			// Token: 0x06000D77 RID: 3447 RVA: 0x0003B0D7 File Offset: 0x000392D7
			set;
		}

		// Token: 0x17000460 RID: 1120
		[HandlerParameter(true, "0", Min = "-100", Max = "100", Step = "1")]
		public int SmoothPhase
		{
			// Token: 0x06000D78 RID: 3448 RVA: 0x0003B0E0 File Offset: 0x000392E0
			get;
			// Token: 0x06000D79 RID: 3449 RVA: 0x0003B0E8 File Offset: 0x000392E8
			set;
		}
	}
}
