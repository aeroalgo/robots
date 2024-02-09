using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000136 RID: 310
	[HandlerCategory("vvRSI"), HandlerDecimals(2), HandlerName("Laguerre")]
	public class Laguerre : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000944 RID: 2372 RVA: 0x00026F04 File Offset: 0x00025104
		public IList<double> Execute(IList<double> src)
		{
			return Laguerre.GenLaguerre(src, this.Gamma, this.Smooth);
		}

		// Token: 0x06000943 RID: 2371 RVA: 0x00026D78 File Offset: 0x00024F78
		public static IList<double> GenLaguerre(IList<double> _source, double _gamma, int _smooth)
		{
			double[] array = new double[_source.Count];
			double num = 0.0;
			double num2 = 0.0;
			double num3 = 0.0;
			double num4 = 0.0;
			double num5 = 0.0;
			for (int i = 0; i < _source.Count; i++)
			{
				double num6 = num;
				double num7 = num2;
				double num8 = num3;
				double num9 = num4;
				num = (1.0 - _gamma) * _source[i] + _gamma * num6;
				num2 = -_gamma * num + num6 + _gamma * num7;
				num3 = -_gamma * num2 + num7 + _gamma * num8;
				num4 = -_gamma * num3 + num8 + _gamma * num9;
				double num10 = 0.0;
				double num11 = 0.0;
				if (num >= num2)
				{
					num10 = num - num2;
				}
				else
				{
					num11 = num2 - num;
				}
				if (num2 >= num3)
				{
					num10 = num10 + num2 - num3;
				}
				else
				{
					num11 = num11 + num3 - num2;
				}
				if (num3 >= num4)
				{
					num10 = num10 + num3 - num4;
				}
				else
				{
					num11 = num11 + num4 - num3;
				}
				if (num10 + num11 != 0.0)
				{
					num5 = num10 / (num10 + num11);
				}
				array[i] = num5 * 100.0;
			}
			IList<double> result = array;
			if (_smooth > 0)
			{
				result = JMA.GenJMA(array, _smooth, 100);
			}
			return result;
		}

		// Token: 0x170002FF RID: 767
		public IContext Context
		{
			// Token: 0x06000945 RID: 2373 RVA: 0x00026F18 File Offset: 0x00025118
			get;
			// Token: 0x06000946 RID: 2374 RVA: 0x00026F20 File Offset: 0x00025120
			set;
		}

		// Token: 0x170002FD RID: 765
		[HandlerParameter(true, "0.7", Min = "0", Max = "1", Step = "0.1")]
		public double Gamma
		{
			// Token: 0x0600093F RID: 2367 RVA: 0x00026D55 File Offset: 0x00024F55
			get;
			// Token: 0x06000940 RID: 2368 RVA: 0x00026D5D File Offset: 0x00024F5D
			set;
		}

		// Token: 0x170002FE RID: 766
		[HandlerParameter(true, "1", Min = "0", Max = "25", Step = "1")]
		public int Smooth
		{
			// Token: 0x06000941 RID: 2369 RVA: 0x00026D66 File Offset: 0x00024F66
			get;
			// Token: 0x06000942 RID: 2370 RVA: 0x00026D6E File Offset: 0x00024F6E
			set;
		}
	}
}
