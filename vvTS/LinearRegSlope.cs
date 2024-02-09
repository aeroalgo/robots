using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000184 RID: 388
	[HandlerCategory("vvAverages"), HandlerName("LinearRegrSlope")]
	public class LinearRegSlope : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000C47 RID: 3143 RVA: 0x00035493 File Offset: 0x00033693
		public IList<double> Execute(IList<double> src)
		{
			return LinearRegSlope.GenLRS(src, this.Length);
		}

		// Token: 0x06000C46 RID: 3142 RVA: 0x00035364 File Offset: 0x00033564
		public static IList<double> GenLRS(IList<double> _source, int _length)
		{
			double[] array = new double[_source.Count];
			IList<double> list = Series.EMA(_source, 1);
			double num = (double)(_length * (_length - 1)) * 0.5;
			double num2 = ((double)_length - 1.0) * (double)_length * (2.0 * (double)_length - 1.0) / 6.0;
			for (int i = 0; i < _source.Count; i++)
			{
				if (i < _length)
				{
					array[i] = 0.0;
				}
				else
				{
					double num3 = 0.0;
					double num4 = 0.0;
					for (int j = 0; j <= _length - 1; j++)
					{
						num3 += (double)j * list[i - j];
						num4 += list[i - j];
					}
					double num5 = num * num4;
					double num6 = (double)_length * num3 - num5;
					double num7 = num * num - (double)_length * num2;
					if (num7 != 0.0)
					{
						array[i] = 100.0 * num6 / num7;
					}
					else
					{
						array[i] = 0.0;
					}
				}
			}
			return array;
		}

		// Token: 0x17000405 RID: 1029
		public IContext Context
		{
			// Token: 0x06000C48 RID: 3144 RVA: 0x000354A1 File Offset: 0x000336A1
			get;
			// Token: 0x06000C49 RID: 3145 RVA: 0x000354A9 File Offset: 0x000336A9
			set;
		}

		// Token: 0x17000404 RID: 1028
		[HandlerParameter(true, "14", Min = "1", Max = "100", Step = "1")]
		public int Length
		{
			// Token: 0x06000C44 RID: 3140 RVA: 0x00035351 File Offset: 0x00033551
			get;
			// Token: 0x06000C45 RID: 3141 RVA: 0x00035359 File Offset: 0x00033559
			set;
		}
	}
}
