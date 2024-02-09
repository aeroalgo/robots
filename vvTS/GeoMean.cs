using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000176 RID: 374
	[HandlerCategory("vvAverages"), HandlerName("Geometric Mean")]
	public class GeoMean : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000BCC RID: 3020 RVA: 0x00032B88 File Offset: 0x00030D88
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("GeometricMean", new string[]
			{
				this.Period.ToString(),
				src.GetHashCode().ToString()
			}, () => GeoMean.GenGeoMean(src, this.Period));
		}

		// Token: 0x06000BCA RID: 3018 RVA: 0x00032AD0 File Offset: 0x00030CD0
		public static IList<double> GenGeoMean(IList<double> src, int period)
		{
			double[] array = new double[src.Count];
			for (int i = 0; i < src.Count; i++)
			{
				if (i < period)
				{
					array[i] = src[i];
				}
				else
				{
					array[i] = GeoMean.iGeoMean(src, period, i);
				}
			}
			return array;
		}

		// Token: 0x06000BCB RID: 3019 RVA: 0x00032B18 File Offset: 0x00030D18
		public static double iGeoMean(IList<double> price, int period, int barNum)
		{
			double num = Math.Pow(price[barNum], 1.0 / (double)period);
			for (int i = 1; i < period; i++)
			{
				num *= Math.Pow(price[barNum - i], 1.0 / (double)period);
			}
			return num;
		}

		// Token: 0x170003E1 RID: 993
		public IContext Context
		{
			// Token: 0x06000BCD RID: 3021 RVA: 0x00032BF4 File Offset: 0x00030DF4
			get;
			// Token: 0x06000BCE RID: 3022 RVA: 0x00032BFC File Offset: 0x00030DFC
			set;
		}

		// Token: 0x170003E0 RID: 992
		[HandlerParameter(true, "15", Min = "7", Max = "30", Step = "1")]
		public int Period
		{
			// Token: 0x06000BC8 RID: 3016 RVA: 0x00032ABC File Offset: 0x00030CBC
			get;
			// Token: 0x06000BC9 RID: 3017 RVA: 0x00032AC4 File Offset: 0x00030CC4
			set;
		}
	}
}
