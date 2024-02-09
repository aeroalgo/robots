using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000045 RID: 69
	[HandlerCategory("vvIndicators"), HandlerName("Polarized Fractal Efficiency")]
	public class PFE : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000277 RID: 631 RVA: 0x0000BD20 File Offset: 0x00009F20
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("PolarizedFractalEfficiency", new string[]
			{
				this.PfePeriod.ToString(),
				this.MaPeriod.ToString(),
				src.GetHashCode().ToString()
			}, () => PFE.GenPFE(src, this.PfePeriod, this.MaPeriod));
		}

		// Token: 0x06000276 RID: 630 RVA: 0x0000BC30 File Offset: 0x00009E30
		public static IList<double> GenPFE(IList<double> src, int pfeperiod, int maperiod)
		{
			int count = src.Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			for (int i = pfeperiod; i < count; i++)
			{
				double num = 1E-09;
				for (int j = 0; j < pfeperiod; j++)
				{
					num += Math.Abs(src[i - j] - src[i - j - 1]);
				}
				array2[i] = (src[i] - src[i - pfeperiod]) / num;
			}
			for (int k = 1; k < count; k++)
			{
				if (maperiod > 0)
				{
					array[k] = EMA.iEMA(array2, array, maperiod, k);
				}
				else
				{
					array[k] = array2[k];
				}
			}
			return array;
		}

		// Token: 0x170000D5 RID: 213
		public IContext Context
		{
			// Token: 0x06000279 RID: 633 RVA: 0x0000BDA7 File Offset: 0x00009FA7
			private get;
			// Token: 0x06000278 RID: 632 RVA: 0x0000BD9E File Offset: 0x00009F9E
			set;
		}

		// Token: 0x170000D4 RID: 212
		[HandlerParameter(true, "5", Min = "2", Max = "20", Step = "1")]
		public int MaPeriod
		{
			// Token: 0x06000274 RID: 628 RVA: 0x0000BC1C File Offset: 0x00009E1C
			get;
			// Token: 0x06000275 RID: 629 RVA: 0x0000BC24 File Offset: 0x00009E24
			set;
		}

		// Token: 0x170000D3 RID: 211
		[HandlerParameter(true, "9", Min = "2", Max = "20", Step = "1")]
		public int PfePeriod
		{
			// Token: 0x06000272 RID: 626 RVA: 0x0000BC0B File Offset: 0x00009E0B
			get;
			// Token: 0x06000273 RID: 627 RVA: 0x0000BC13 File Offset: 0x00009E13
			set;
		}
	}
}
