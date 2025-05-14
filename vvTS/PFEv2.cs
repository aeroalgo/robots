using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000046 RID: 70
	[HandlerCategory("vvIndicators"), HandlerName("Polarized Fractal Efficiency v2")]
	public class PFEv2 : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000280 RID: 640 RVA: 0x0000BF70 File Offset: 0x0000A170
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("PFEv2", new string[]
			{
				this.PfePeriod.ToString(),
				this.smooth.ToString(),
				src.GetHashCode().ToString()
			}, () => PFEv2.GenPFEv2(src, this.PfePeriod, this.smooth));
		}

		// Token: 0x0600027F RID: 639 RVA: 0x0000BDDC File Offset: 0x00009FDC
		public static IList<double> GenPFEv2(IList<double> src, int _pfeperiod, int _smooth)
		{
			int count = src.Count;
			double[] array = new double[count];
			int len = Math.Max(_smooth, 1);
			for (int i = _pfeperiod; i < count; i++)
			{
				if (i < _pfeperiod)
				{
					array[i] = 1E-06;
				}
				else
				{
					double num = 0.0;
					double num2 = Math.Sqrt(Math.Pow(src[i] - src[i - _pfeperiod], 2.0) + Math.Pow((double)_pfeperiod, 2.0));
					for (int j = 1; j <= _pfeperiod; j++)
					{
						num += Math.Sqrt(Math.Pow(src[i - j + 1] - src[i - j], 2.0) + 1.0);
					}
					double num3 = src[i] - src[i - _pfeperiod];
					if (num3 > 0.0)
					{
						array[i] = 1.0 * num2 / num * 100.0;
					}
					else if (num3 < 0.0)
					{
						array[i] = -1.0 * num2 / num * 100.0;
					}
					else
					{
						array[i] = array[i - 1];
					}
				}
			}
			return JMA.GenJMA(array, len, 0);
		}

		// Token: 0x170000D8 RID: 216
		public IContext Context
		{
			// Token: 0x06000282 RID: 642 RVA: 0x0000BFF7 File Offset: 0x0000A1F7
			private get;
			// Token: 0x06000281 RID: 641 RVA: 0x0000BFEE File Offset: 0x0000A1EE
			set;
		}

		// Token: 0x170000D6 RID: 214
		[HandlerParameter(true, "9", Min = "2", Max = "20", Step = "1")]
		public int PfePeriod
		{
			// Token: 0x0600027B RID: 635 RVA: 0x0000BDB7 File Offset: 0x00009FB7
			get;
			// Token: 0x0600027C RID: 636 RVA: 0x0000BDBF File Offset: 0x00009FBF
			set;
		}

		// Token: 0x170000D7 RID: 215
		[HandlerParameter(true, "5", Min = "2", Max = "20", Step = "1")]
		public int smooth
		{
			// Token: 0x0600027D RID: 637 RVA: 0x0000BDC8 File Offset: 0x00009FC8
			get;
			// Token: 0x0600027E RID: 638 RVA: 0x0000BDD0 File Offset: 0x00009FD0
			set;
		}
	}
}
