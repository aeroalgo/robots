using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200003B RID: 59
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("Regularized Momentum")]
	public class MomentumReg : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000222 RID: 546 RVA: 0x00009EB8 File Offset: 0x000080B8
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("momentumregul", new string[]
			{
				this.MomPeriod.ToString(),
				this.Lambda.ToString(),
				src.GetHashCode().ToString()
			}, () => MomentumReg.GenMomentumReg(src, this.MomPeriod, this.Lambda));
		}

		// Token: 0x06000221 RID: 545 RVA: 0x00009DA4 File Offset: 0x00007FA4
		public static IList<double> GenMomentumReg(IList<double> src, int _momperiod, int _Lambda)
		{
			double[] array = new double[src.Count];
			double[] array2 = new double[src.Count];
			double num = 2.0 / (1.0 * (double)_momperiod);
			double num2 = 1.0 + (double)_Lambda * 2.0;
			double num3 = 1.0 + (double)_Lambda;
			array2[0] = src[0];
			array[0] = 0.0;
			for (int i = 1; i < src.Count; i++)
			{
				if (i > 2)
				{
					array2[i] = (num2 * array2[i - 1] + num * (src[i] - array2[i - 1]) - (double)_Lambda * array2[i - 2]) / num3;
				}
				else
				{
					array2[i] = src[i];
				}
				array[i] = (array2[i] - array2[i - 1]) / array2[i] * 100.0;
			}
			return array;
		}

		// Token: 0x170000B9 RID: 185
		public IContext Context
		{
			// Token: 0x06000223 RID: 547 RVA: 0x00009F36 File Offset: 0x00008136
			get;
			// Token: 0x06000224 RID: 548 RVA: 0x00009F3E File Offset: 0x0000813E
			set;
		}

		// Token: 0x170000B8 RID: 184
		[HandlerParameter(true, "5", Min = "0", Max = "20", Step = "1")]
		public int Lambda
		{
			// Token: 0x0600021F RID: 543 RVA: 0x00009D92 File Offset: 0x00007F92
			get;
			// Token: 0x06000220 RID: 544 RVA: 0x00009D9A File Offset: 0x00007F9A
			set;
		}

		// Token: 0x170000B7 RID: 183
		[HandlerParameter(true, "10", Min = "2", Max = "20", Step = "1")]
		public int MomPeriod
		{
			// Token: 0x0600021D RID: 541 RVA: 0x00009D81 File Offset: 0x00007F81
			get;
			// Token: 0x0600021E RID: 542 RVA: 0x00009D89 File Offset: 0x00007F89
			set;
		}
	}
}
