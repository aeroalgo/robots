using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200012B RID: 299
	[HandlerCategory("vvRSI"), HandlerName("Accumulated RSI")]
	public class RSIacc : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060008B7 RID: 2231 RVA: 0x00024A86 File Offset: 0x00022C86
		public IList<double> Execute(IList<double> src)
		{
			return RSIacc.GenRSIacc(src, this.RSIperiod, this.ACCperiod, this.Context);
		}

		// Token: 0x060008B6 RID: 2230 RVA: 0x000249C8 File Offset: 0x00022BC8
		public static IList<double> GenRSIacc(IList<double> src, int _RSIperiod, int _ACCperiod, IContext context)
		{
			int count = src.Count;
			double[] array = new double[count];
			IList<double> data = context.GetData("rsi", new string[]
			{
				_RSIperiod.ToString(),
				src.GetHashCode().ToString()
			}, () => Series.RSI(src, _RSIperiod));
			for (int i = _ACCperiod; i < count; i++)
			{
				for (int j = 0; j < _ACCperiod; j++)
				{
					array[i] += data[i - j];
				}
			}
			return array;
		}

		// Token: 0x170002C9 RID: 713
		[HandlerParameter(true, "3", Min = "3", Max = "3", Step = "0")]
		public int ACCperiod
		{
			// Token: 0x060008B4 RID: 2228 RVA: 0x0002499B File Offset: 0x00022B9B
			get;
			// Token: 0x060008B5 RID: 2229 RVA: 0x000249A3 File Offset: 0x00022BA3
			set;
		}

		// Token: 0x170002CA RID: 714
		public IContext Context
		{
			// Token: 0x060008B8 RID: 2232 RVA: 0x00024AA0 File Offset: 0x00022CA0
			get;
			// Token: 0x060008B9 RID: 2233 RVA: 0x00024AA8 File Offset: 0x00022CA8
			set;
		}

		// Token: 0x170002C8 RID: 712
		[HandlerParameter(true, "2", Min = "2", Max = "2", Step = "0")]
		public int RSIperiod
		{
			// Token: 0x060008B2 RID: 2226 RVA: 0x0002498A File Offset: 0x00022B8A
			get;
			// Token: 0x060008B3 RID: 2227 RVA: 0x00024992 File Offset: 0x00022B92
			set;
		}
	}
}
